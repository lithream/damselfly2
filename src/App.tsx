import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import Graph from "./GraphComponent";
import MapGrid from "./MapGridComponent";
import OperationLog from "./OperationLogComponent.tsx";
import GraphSlider from "./GraphSliderComponent.tsx";
import Callstack from "./CallstackComponent.tsx";
import Data from "./Data.tsx";
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

function App() {
  const [dataLoaded, setDataLoaded] = useState<boolean>(false);
  const [xClick, setXClick] = useState<number>(0);
  const [xLimit, setXLimit] = useState<number>(0);
  const [memoryData, setMemoryData] = useState<Data>({ timestamp: 0, data: [] });
  const [blockSize, setBlockSize] = useState<number>(32);

  useEffect(() => {
    const fetchData = async () => {
      if (dataLoaded) {
        try {
          const data: [number, number[][]] = await invoke("get_viewer_map_full_at_colours", { timestamp: xClick, truncateAfter: 256 });
          const memoryData: Data = {
            timestamp: data[0],
            data: data[1],
          };
          console.log("memory data set");
          setMemoryData(memoryData);
        } catch (error) {
          console.log("error");
          console.error("Error fetching memory data: ", error);
        }
      }
    };
    fetchData().then();
  }, [xClick, dataLoaded, blockSize]);

  const selectFilesAndInitialiseViewer = async () => {
    try {
      const logFilePath = await invoke("choose_files");
      const binaryFilePath = await invoke("choose_files");

      if (logFilePath && binaryFilePath) {
        await invoke("initialise_viewer", { log_path: logFilePath, binary_path: binaryFilePath });
        setDataLoaded(true);
      }
    } catch (error) {
      console.error("Error initialising viewer: ", error);
    }
  }

  const increaseBlockSize = async () => {
    setBlockSize(blockSize * 2);
    await invoke("set_block_size", { newBlockSize: Math.ceil(blockSize) });
  };

  const decreaseBlockSize = async () => {
    if (blockSize <= 1) {
      setBlockSize(1);
      return;
    }
    setBlockSize(blockSize / 2);
    await invoke("set_block_size", { newBlockSize: Math.ceil(blockSize) });
  }

  return (
    <div className="container">
      <div className="mainContent">
        <div className="left">
          <div className="top">
            <Graph dataLoaded={dataLoaded} setXClick={setXClick} xClick={xClick} setXLimit={setXLimit} />
            <GraphSlider xClick={xClick} setXClick={setXClick} xLimit={xLimit}/>
          </div>
          <div className="bottom">
            <OperationLog memoryData={memoryData} dataLoaded={dataLoaded} xClick={xClick} />
            <Callstack xClick={xClick}/>
          </div>
        </div>
        <div className="right">
          <MapGrid memoryData={memoryData} blockSize={4}></MapGrid>
        </div>
      </div>
      <div className="controlPanel">
        <div className="buttonGroup">
          <button onClick={selectFilesAndInitialiseViewer}>Load</button>
          <button onClick={() => increaseBlockSize()}>+</button>
          <button onClick={() => decreaseBlockSize()}>-</button>
        </div>
        <div className="memoryStateLegend">
            <div className="legend-item">
              <div className="legend-square" style={{ backgroundColor: 'red' }}></div>
              <span className="legend-text">ALLOCATED</span>
            </div>
            <div className="legend-item">
              <div className="legend-square" style={{ backgroundColor: 'yellow' }}></div>
              <span className="legend-text">PARTIALLY ALLOCATED</span>
            </div>
            <div className="legend-item">
              <div className="legend-square" style={{ backgroundColor: 'grey' }}></div>
              <span className="legend-text">FREE</span>
            </div>
          </div>
        </div>
      </div>
  );
}

export default App;

