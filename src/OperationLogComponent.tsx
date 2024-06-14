import {invoke} from "@tauri-apps/api/tauri";
import {useEffect, useState} from "react";
import Data from "./Data.tsx";

interface OperationLogProps {
    activeInstance: number;
    memoryData: Data;
    dataLoaded: boolean;
    xClick: number;
    setSelectedBlock: (block: number) => void;
    setLookupTile: (block: number) => void;
    setSelectedTile: (block: number) => void;
    setRealtimeGraph: (realtime: boolean) => void;
    setXClick: (x: number) => void;
}

function OperationLog({ activeInstance, memoryData, setSelectedBlock, setLookupTile, setSelectedTile, setRealtimeGraph, setXClick }: OperationLogProps) {
    const [log, setLog] = useState<string[]>([]);
    useEffect(() => {
        const fetchLog = async () => {
            try {
                const fetchedLog = await invoke<string[]>("get_operation_log", {
                    damselflyInstance: activeInstance
                });
                setLog(fetchedLog);
            } catch (error) {
                console.error("Failed to fetch operation log", error);
            }
        }
        fetchLog().then();
    }, [memoryData]);

    const handleLogEntryClick = (logEntry: string) => {
        const addressPattern = /0x[0-9a-fA-F]+/;
        const optimePattern = /\[(\d+)\s/;
        
        const addressMatch = logEntry.match(addressPattern);
        const optimeMatch = logEntry.match(optimePattern);

        if (addressMatch && optimeMatch) {
            const address = parseInt(addressMatch[0], 16);
            const optime = parseInt(optimeMatch[1], 10); // optime is parsed as a base 10 integer

            setSelectedBlock(address);
            setLookupTile(address);
            setSelectedTile(-1);
            setRealtimeGraph(false);
            setXClick(optime);
        }
    };

    return (
        <div className="log-container">
            {log.map((entry, index) => (
                <div key={index} className="log-entry" onClick={() => handleLogEntryClick(entry)}>{entry}</div>
            ))}
        </div>
    )
}

export default OperationLog;