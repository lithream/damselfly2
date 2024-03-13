import {invoke} from "@tauri-apps/api/tauri";
import {useEffect, useState} from "react";

type Data = {
    timestamp: number;
    data: number[];
}

interface OperationLogProps {
    memoryData: Data;
    dataLoaded: boolean;
    xClick: number;
}

function OperationLog({ memoryData }: OperationLogProps) {
    const [log, setLog] = useState<string[]>([]);
    useEffect(() => {
        const fetchLog = async () => {
            try {
                const fetchedLog = await invoke<string[]>("get_operation_log");
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