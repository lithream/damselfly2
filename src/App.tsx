import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import Graph from "./GraphComponent";
import MapGrid from "./MapGridComponent";
import OperationLog from "./OperationLogComponent.tsx";
import GraphSlider from "./GraphSliderComponent.tsx";
import Callstack from "./CallstackComponent.tsx";
import BlockStatus from "./BlockStatusComponent.tsx";
import Data from "./Data.tsx";
import '@fontsource/roboto/300.css';
import '@fontsource/roboto/400.css';
import '@fontsource/roboto/500.css';
import '@fontsource/roboto/700.css';

function App() {
  const [dataLoaded, setDataLoaded] = useState<boolean>(false);
  const [xClick, setXClick] = useState<number>(0);
  const [selectedBlock, setSelectedBlock] = useState<number>(0);
  const [xLimit, setXLimit] = useState<number>(0);
  const [realtimeGraph, setRealtimeGraph] = useState<boolean>(true);
  const [realtimeGraphOffset, setRealtimeGraphOffset] = useState<number>(0);
  const [memoryData, setMemoryData] = useState<Data>({ timestamp: 0, data: [] });
  const [blockSize, setBlockSize] = useState<number>(32);
  const [squareSize, setSquareSize] = useState<number>(4);
  const [activeTab, setActiveTab] = useState('callstack');

  useEffect(() => {
    const fetchData = async () => {
      if (dataLoaded) {
        try {
          let data: [number, number[][]];
          if (realtimeGraph) {
            data = await invoke("get_viewer_map_full_at_colours_realtime_sampled", {
              timestamp: xClick + realtimeGraphOffset,
              truncateAfter: 256
            });
          } else {
            data = await invoke("get_viewer_map_full_at_colours", {
              timestamp: xClick,
              truncateAfter: 256
            });
          }

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
      let cacheSize = prompt("Enter cache size. Smaller caches run faster but use more memory.\n" +
          "Defaults to 1000 if blank, which is suitable for logs up to 50MB.");
      if (cacheSize == null) {
        cacheSize = "1000";
      }
      console.log(`cacheSize = ${cacheSize}`);
      let cacheSizeInt = parseInt(cacheSize);
      if (isNaN(cacheSizeInt)) {
        cacheSizeInt = 1000;
      }
      console.log(`cacheSizeInt = ${cacheSizeInt}`);
      const logFilePath = await invoke("choose_files");
      const binaryFilePath = await invoke("choose_files");

      if (logFilePath && binaryFilePath) {
        await invoke("initialise_viewer", { log_path: logFilePath, binary_path: binaryFilePath, cache_size: cacheSizeInt });
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

  const increaseSquareSize = async () => {
    setSquareSize(squareSize + 4);
  }

  const decreaseSquareSize = async () => {
    let newSquareSize = squareSize - 4;
    if (newSquareSize <= 0) {
      newSquareSize = 4;
    }
    setSquareSize(newSquareSize);
  }

  const toggleRealtime = async () => {
    setXClick(0);
    setRealtimeGraph(!realtimeGraph);
  }

  return (
    <div className="container">
      <div className="mainContent">
        <div className="left">
          <div className="top">
            <Graph dataLoaded={dataLoaded} realtimeGraph={realtimeGraph} setXClick={setXClick} xClick={xClick} setXLimit={setXLimit} setRealtimeGraphOffset={setRealtimeGraphOffset} />
            <GraphSlider xClick={xClick} setXClick={setXClick} xLimit={xLimit}/>
          </div>
          <div className="tabs">
            <button onClick={() => setActiveTab('operationLog')} className={activeTab === 'operationLog' ? 'active' : ''}>Operation Log</button>
            <button onClick={() => setActiveTab('callstack')} className={activeTab === 'callstack' ? 'active' : ''}>Callstack</button>
            <button onClick={() => setActiveTab('block')} className={activeTab === 'block' ? 'active' : ''}>Block</button>
          </div>
          <div className="tabContent">
            {activeTab === 'operationLog' && <OperationLog memoryData={memoryData} dataLoaded={dataLoaded} xClick={xClick} />}
            {activeTab === 'callstack' && <Callstack xClick={xClick} />}
            {activeTab === 'block' && <BlockStatus selectedBlock={selectedBlock} timestamp={realtimeGraph ? xClick + realtimeGraphOffset : xClick} realtimeGraph={realtimeGraph}/>}
          </div>
          <div className="bottom">
            {/* GraphSlider or other components if needed */}
          </div>
        </div>
        <div className="right">
          <MapGrid memoryData={memoryData} blockSize={4} squareSize={squareSize} selectedBlock={selectedBlock} setSelectedBlock={setSelectedBlock}></MapGrid>
        </div>
      </div>
      <div className="controlPanel">
        <div className="buttonGroup">
          <button onClick={selectFilesAndInitialiseViewer}>Load</button>
          <button onClick={() => increaseBlockSize()}>+</button>
          <button onClick={() => decreaseBlockSize()}>-</button>
          <button onClick={() => toggleRealtime()}>TIME</button>
          <button onClick={() => increaseSquareSize()}>+</button>
          <button onClick={() => decreaseSquareSize()}>-</button>
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

