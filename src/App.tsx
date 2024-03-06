import { useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import "./App.css";

function App() {
  const [greetMsg, setGreetMsg] = useState("");
  const [name, setName] = useState("");

  const selectFilesAndInitialiseViewer = async () => {
    try {
      const logFilePath = await invoke("choose_files");
      const binaryFilePath = await invoke("choose_files");
      console.log(logFilePath);
      console.log(binaryFilePath);

      if (logFilePath && binaryFilePath) {
        await invoke("initialise_viewer", { log_path: logFilePath, binary_path: binaryFilePath });
      }
    } catch (error) {
      console.error("Error initialising viewer: ", error);
    }
  }

  return (
    <div className="container">
      <h1>Welcome to Tauri!</h1>
      <div className="controlPanel">
        <button onClick={selectFilesAndInitialiseViewer}>Load</button>
      </div>
    </div>
  );
}

export default App;
