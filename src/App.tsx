import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import Graph from "./GraphComponent";
import MapGrid from "./MapGridComponent";

// Assuming the memory block status type is already defined somewhere
type BlockStatus = "Allocated" | "PartiallyAllocated" | "Free" | "Unused";

function App() {
  const [dataLoaded, setDataLoaded] = useState<boolean>(false);
  const [xClick, setXClick] = useState<number>(0);
  const [xHover, setXHover] = useState<number>(0);
  const [memoryData, setMemoryData] = useState<BlockStatus[]>([]); 
  const []

  useEffect(() => {
    const fetchData = async () => {
      if (dataLoaded) {
        try {
          if (xHover > -1) {
            const data: BlockStatus[] = await invoke("get_viewer_map_full_at", { timestamp: xHover });
            setMemoryData(data);
          } else {
            const data: BlockStatus[] = await invoke("get_viewer_map_full_at", { timestamp: xClick });
            setMemoryData(data);
          }
        } catch (error) {
          console.error("Error fetching memory data: ", error);
        }
      }
    };
    fetchData();
  }, [xClick, xHover, dataLoaded]); // Depend on xChangedTrigger and dataLoadedTrigger

  const selectFilesAndInitialiseViewer = async () => {
    try {
      const logFilePath = await invoke("choose_files");
      const binaryFilePath = await invoke("choose_files");
      console.log(logFilePath);
      console.log(binaryFilePath);

      if (logFilePath && binaryFilePath) {
        await invoke("initialise_viewer", { log_path: logFilePath, binary_path: binaryFilePath });
        setDataLoaded(true); // Set to true directly since we're initializing
      }
    } catch (error) {
      console.error("Error initialising viewer: ", error);
    }
  }

  return (
    <div className="container">
      <div className="top">
        <Graph dataLoaded={dataLoaded}
         setXClick={setXClick}
         setXHover={setXHover}
          />
      </div>
      <div className="bottom">
        <MapGrid data={memoryData}/>
      </div>
      <div className="controlPanel">
        <button onClick={selectFilesAndInitialiseViewer}>Load</button>
      </div>
    </div>
  );
}

export default App;

