import {invoke} from "@tauri-apps/api/tauri";
import {useEffect, useState} from "react";

interface OperationLogProps {
    dataLoaded: boolean;
    xClick: number;
    xHover: number;
}

function OperationLog({ dataLoaded, xClick, xHover }: OperationLogProps) {
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
        if (dataLoaded) {
            fetchLog();
        }
    }, [dataLoaded, xClick, xHover]);

    return (
        <div className="log-container">
            {log.map((entry, index) => (
                <div key={index} className="log-entry">{entry}</div>
            ))}
        </div>
    )
}

export default OperationLog;