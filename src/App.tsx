import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import Graph from "./GraphComponent";
import MapGrid from "./MapGridComponent";

type Data = {
  timestamp: number;
  data: number[];
}

function App() {
  const [dataLoaded, setDataLoaded] = useState<boolean>(false);
  const [xClick, setXClick] = useState<number>(0);
  const [xHover, setXHover] = useState<number>(0);
  const [memoryData, setMemoryData] = useState<Data>({ timestamp: 0, data: [] });

  useEffect(() => {
    const fetchData = async () => {
      if (dataLoaded) {
        try {
          if (xHover > -1) {
            const data: Data = await invoke("get_viewer_map_full_at_colours", { timestamp: xHover, truncateAfter: 256 });
            setMemoryData(data);
          } else {
            const data: Data = await invoke("get_viewer_map_full_at_colours", { timestamp: xClick, truncateAfter: 256 });
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
        setDataLoaded(true);
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
        <MapGrid data={memoryData}></MapGrid>
      </div>
      <div className="controlPanel">
        <button onClick={selectFilesAndInitialiseViewer}>Load</button>
      </div>
    </div>
  );
}

export default App;

