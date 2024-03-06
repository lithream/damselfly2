import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";
import Graph from "./GraphComponent";

function App() {
  const [dataLoadedTrigger, setDataLoadedTrigger] = useState(false);

  const selectFilesAndInitialiseViewer = async () => {
    try {
      const logFilePath = await invoke("choose_files");
      const binaryFilePath = await invoke("choose_files");
      console.log(logFilePath);
      console.log(binaryFilePath);

      if (logFilePath && binaryFilePath) {
        await invoke("initialise_viewer", { log_path: logFilePath, binary_path: binaryFilePath });
        setDataLoadedTrigger(prev => !prev);
      }
    } catch (error) {
      console.error("Error initialising viewer: ", error);
    }
  }

  return (
    <div className="container">
      <div className="controlPanel">
        <button onClick={selectFilesAndInitialiseViewer}>Load</button>
      </div>
      <Graph dataLoadedTrigger={dataLoadedTrigger} />
    </div>
  );
}

export default App;
