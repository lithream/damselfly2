import {invoke} from "@tauri-apps/api/tauri";
import {useEffect, useState} from "react";
import Data from "./Data.tsx";

interface OperationLogProps {
    activeInstance: number;
    memoryData: Data;
    dataLoaded: boolean;
    xClick: number;
}

function OperationLog({ activeInstance, memoryData }: OperationLogProps) {
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

    return (
        <div className="log-container">
            {log.map((entry, index) => (
                <div key={index} className="log-entry">{entry}</div>
            ))}
        </div>
    )
}

export default OperationLog;