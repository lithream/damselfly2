from jinja2 import Environment, FileSystemLoader
from pprint import pprint
import subprocess
import sys
import json
import math
import time
import os


class LineParser:
    def __init__(self, line):
        self.data_dict = {}
        self.parseMemInfoLine(line)

    def getDataDict(self):
        return self.data_dict

    def parseMemInfoLine(self, line):
        """
        Input: a line from check log
        Output: a dict with information(timestamp, mem address, return add...)
        """

        # check timestamp
        splittedLine = line.strip().split()
        try:
            timestamp = float(splittedLine[5])
            unit = splittedLine[6]
            if unit == "us":
                timestamp /= 1000000
            elif unit == "ms":
                timestamp /= 1000
            # dart trace
            dataLine = line.split('>')[1].strip()
        except:
            try:
                # without prefix
                timestamp = float(splittedLine[0])
            except:
                try:
                    # with prefix
                    timestamp = float(splittedLine[0].split(':')[1])
                except:
                    return
            try:
                # sift log
                dataLine = line.split(')')[1].strip()
            except:
                return

        splittedLine = dataLine.split(' ')
        # skip irrelavant line
        if len(splittedLine) < 2 or len(splittedLine) > 3:
            return

        if splittedLine[0] == "+":
            rec_type = "allocate"
        elif splittedLine[0] == "-":
            rec_type = "free"
        elif splittedLine[0] == "^":
            rec_type = "trace"
        else:
            return

        try:
            mem_address = "0x" + splittedLine[1]
            if rec_type == "allocate":
                mem_size = int(splittedLine[2], 16)
            elif rec_type == "trace":
                ret_address = "0x" + splittedLine[2].replace("[", "").replace("]", "")
        except:
            print("Warning: skip incorrectly decoded line (" + line + ")", file=sys.stderr)
            return

        # everything is fine, update the dict
        self.data_dict["time stamp"] = timestamp
        self.data_dict["type"] = rec_type
        self.data_dict["memory address"] = mem_address
        try:
            self.data_dict["memory size"] = mem_size
        except:
            pass
        try:
            self.data_dict["return address"] = ret_address
        except:
            pass


class MemLeakAnalyzer:
    def __init__(self, checklog_file_path, output_file_path, main_debug_out_file, time_interval):
        # "memory leak": addresses that were allocated but never been decallocated
        # "allocation before deallocation": addresses that were allocated 2 consecutive times without a deallocation in between
        # "deallocation before allocation": addresses that were deallocated 2 consecutive times without an allocation in between
        self.output_dicts = {"memory leak": {}, "allocation before deallocation": {},
                             "deallocation before allocation": {}}

        # a time series of memory pool states
        self.memory_pool_states = []
        self.snapshot_moments = []

        self.checklog_file_path = checklog_file_path
        self.output_file_path = output_file_path
        self.main_debug_out_file = main_debug_out_file
        self.time_interval = time_interval

    def initialize(self):
        # calculate the number of lines in the file
        print("Initializing...")
        with open(self.checklog_file_path, 'r') as file:
            self.total_lines = sum(1 for line in file)

    def analyze(self):
        start_time = time.time()

        self.initialize()
        self.parseCheckLog()
        # self.outputToTxtFile()

        print(f"Execution time: {time.time() - start_time}")

    def parseCheckLog(self):
        print("Analyzing...")

        end_time = -1

        with open(self.checklog_file_path, 'r') as file:
            for i, line in enumerate(file, 1):
                print(f'{i}/{self.total_lines} lines', end='\r')
                lineParser = LineParser(line)
                data_dict = lineParser.getDataDict().copy()

                # skip lines failed to parse
                if not data_dict:
                    continue

                # this code is for taking a snapshot of memory pool
                if data_dict["type"] == "allocate" or data_dict["type"] == "free":
                    # current data_dict is the point beyond the end_time
                    # a snapshot of memory pool at end_time should be taken
                    incoming_timestamp = data_dict["time stamp"]
                    if incoming_timestamp > end_time:
                        # keep incoming_timestamp intact if time_interval==0 (i.e. plot all data)
                        if self.time_interval > 0:
                            incoming_timestamp = incoming_timestamp // self.time_interval * self.time_interval
                            if incoming_timestamp == data_dict["time stamp"]:
                                incoming_timestamp -= self.time_interval

                        # the first data point, at this point the memory usage = 0
                        if end_time == -1:
                            if self.time_interval == 0:
                                # if incoming_timestamp = 2.5 => the first time stamp is 2
                                # if incoming_timestamp = 2 => the first time stamp is 1
                                self.snapshot_moments.append(math.ceil(incoming_timestamp - 1))
                            else:
                                self.snapshot_moments.append(incoming_timestamp)
                            self.memory_pool_states.append({})
                        else:
                            # the current snapshot
                            self.snapshot_moments.append(end_time)
                            self.memory_pool_states.append(self.output_dicts["memory leak"].copy())

                            # Eg: end_time = 20, incoming_timestamp = 25,
                            # there should be a horizontal line of the graph from time 20->25
                            if self.time_interval != 0 and incoming_timestamp > end_time:
                                self.snapshot_moments.append(incoming_timestamp)
                                self.memory_pool_states.append(self.memory_pool_states[-1].copy())

                        end_time = incoming_timestamp + self.time_interval

                mem_address = data_dict["memory address"]

                if data_dict["type"] == "allocate":
                    data_dict["calling stack"] = list()

                    # check if the alloc mem address is in the dict
                    if self.output_dicts["memory leak"].get(mem_address) is not None:
                        self.output_dicts["allocation before deallocation"][mem_address] = data_dict
                    else:
                        # add new memory address
                        self.output_dicts["memory leak"][mem_address] = data_dict

                elif data_dict["type"] == "free":
                    data_dict["calling stack"] = list()

                    # check if the alloc mem address is not in the dict
                    if self.output_dicts["memory leak"].get(mem_address) is None:
                        self.output_dicts["deallocation before allocation"][mem_address] = data_dict
                    else:
                        # delete memory address
                        self.output_dicts["memory leak"].pop(mem_address)

                elif data_dict["type"] == "trace":
                    if self.output_dicts["memory leak"].get(mem_address) is not None:
                        self.output_dicts["memory leak"][mem_address]["calling stack"].append(
                            {"return address": data_dict["return address"]})
                    elif self.output_dicts["allocation before deallocation"].get(mem_address) is not None:
                        self.output_dicts["allocation before deallocation"][mem_address]["calling stack"].append(
                            {"return address": data_dict["return address"]})
                    elif self.output_dicts["deallocation before allocation"].get(mem_address) is not None:
                        self.output_dicts["deallocation before allocation"][mem_address]["calling stack"].append(
                            {"return address": data_dict["return address"]})

                del lineParser

            # taking the last snapshot of memory pool after the for loop through the checklog file
            self.memory_pool_states.append(self.output_dicts["memory leak"].copy())
            self.snapshot_moments.append(end_time)

        print()

        # for key,value in self.output_dicts.items():
        #     value = dict(sorted(value.items(), key=lambda x:x[1]["time stamp"]))
        #     self.updateInfoFromReturnAddress(value)

        self.updateInfoFromReturnAddressForMemoryPoolState()

    def updateInfoFromReturnAddressForMemoryPoolState(self):
        # contains unique return address
        print("Running gaddr2line...")

        return_addr_set = set()
        for memory_pool_state in self.memory_pool_states:
            for mem_addr, data_dict in memory_pool_state.items():
                for return_addr_dict in data_dict["calling stack"]:
                    return_addr_set.add(return_addr_dict["return address"])

        # key is return address, value is a dictionary of info: file name, func name, line num
        return_addr_info_dict = self.parseReturnAddress(return_addr_set)

        for memory_pool_state in self.memory_pool_states:
            for mem_addr, data_dict in memory_pool_state.items():
                for return_addr_dict in data_dict["calling stack"]:
                    return_addr_dict.update(return_addr_info_dict[return_addr_dict["return address"]])

    def parseReturnAddress(self, return_addr_set):
        """
        Input: a list of return address, Output: a dictionary with key being the return address and value being a dict of file name, function name, and line number
        """
        return_addr_info_dict = {}
        # runnung only 1 command helps save a lot of time 
        cmd_str = "/sirius/tools/ghs/arm2017.5.4a/gaddr2line -e {main_debug_out_file} -f -s ".format(
            main_debug_out_file=self.main_debug_out_file)
        for return_addr in return_addr_set:
            cmd_str += return_addr + " "

        output = subprocess.getoutput(cmd_str)
        output_arr = output.split("\n")

        id = 0
        for return_addr in return_addr_set:
            function_name = output_arr[id].strip()
            file_name = output_arr[id + 1].split(":")[0].strip()
            line_num = output_arr[id + 1].split(":")[1].strip()
            return_addr_info_dict[return_addr] = {'file name': file_name, 'function name': function_name,
                                                  'line number': line_num}
            id += 2

        return return_addr_info_dict

    def debug(self, data):
        with open(self.output_file_path, 'w') as f:
            original_stdout = sys.stdout
            sys.stdout = f
            pprint(data)
            sys.stdout = original_stdout

    def outputToTxtFile(self):
        print("Outputing...")
        mem_leak = self.reformatCallingStack(self.output_dicts["memory leak"])
        alloc_before_delloc = self.reformatCallingStack(self.output_dicts["allocation before deallocation"])
        dealloc_before_alloc = self.reformatCallingStack(self.output_dicts["deallocation before allocation"])
        with open(self.output_file_path, 'w') as f:
            original_stdout = sys.stdout
            sys.stdout = f
            print(
                "----------------------------Memory leak (memory allocated but never been deallocated):----------------------------")
            pprint(mem_leak)
            print(
                "--------------------Memory allocated 2 consecutive times without being deallocated in between:--------------------")
            pprint(alloc_before_delloc)
            print(
                "------------------------------------Memory deallocated before being allocated:------------------------------------")
            pprint(dealloc_before_alloc)
            json.dump(mem_leak, f)
            sys.stdout = original_stdout

    def reformatCallingStack(self, output_dict):

        output_list = []
        for mem_addr, data_dict in output_dict.items():
            new_data_dict = data_dict.copy()
            old_calling_stack = new_data_dict.pop('calling stack')
            new_calling_stack = []
            for calling_function_info in old_calling_stack:
                new_calling_stack.append(
                    calling_function_info["return address"] + "/" + calling_function_info["function name"] + "/" +
                    calling_function_info["file name"] + ":" + calling_function_info["line number"])
            new_data_dict["calling stack"] = new_calling_stack
            output_list.append(new_data_dict)
        return output_list

    def getMemoryPoolStatesForHTML(self):
        # a list of:  {"total memory size":0, "time stamp":1, "allocations":[{"memory address": "0xc0cd3594", "memory size": 76, "time stamp": 3.096773, "calling stack": ["0xe062df30/_tx_thread_sleep/tx_thread_sleep.c:119"]]]}
        result = []

        for i, memory_pool_state in enumerate(self.memory_pool_states):
            total_mem_size = 0
            for mem_addr, data_dict in memory_pool_state.items():
                total_mem_size += data_dict["memory size"]
            allocations = self.reformatCallingStack(memory_pool_state)
            result.append({"total memory size": total_mem_size, "time stamp": self.snapshot_moments[i],
                           "allocations": allocations})

        return result

    def generate_html_file(self, directory):
        # Load the template
        env = Environment(loader=FileSystemLoader(os.path.dirname(os.path.realpath(__file__))))
        template = env.get_template('index.html')

        # Render the template with the data
        rendered_html = template.render(data=self.getMemoryPoolStatesForHTML(), time_interval=self.time_interval)

        # Write the rendered HTML to a file
        with open(directory + '/mtrace.html', 'w') as file:
            file.write(rendered_html)

        print("successfully create a file: mtrace.html")


if __name__ == '__main__':
    # should be /work/user_name/sirius/utils/src/mtrace_checkpoint/src/script
    current_directory = os.getcwd()

    if (len(sys.argv) < 3):
        print(
            "Usage: python3 mtrace_checkpoint.py <checklog_file_path> <main_debug.out> <time_interval_to_sample data>")
    # location of the check log file
    checklog_file_path = sys.argv[1]

    main_debug_out_file = sys.argv[2]
    time_interval = 1
    if len(sys.argv) >= 4:
        time_interval = float(sys.argv[3])

    # only used when running mem_leak_analyzer.outputToTxtFile()
    # the location of the output file
    output_file_path = current_directory + '/out.txt'

    # location of sirius
    # should be /work/user_name/
    directories = current_directory.split('/')
    main_directory = '/'.join(directories[0:3])

    mem_leak_analyzer = MemLeakAnalyzer(checklog_file_path=checklog_file_path, output_file_path=output_file_path,
                                        main_debug_out_file=main_debug_out_file, time_interval=time_interval)
    mem_leak_analyzer.analyze()
    mem_leak_analyzer.generate_html_file(directory=current_directory)
