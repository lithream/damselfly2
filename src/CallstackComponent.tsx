import {useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";

interface CallstackProps {
    activeInstance: number,
    xClick: number
}

function Callstack({ activeInstance, xClick }: CallstackProps) {
    const [callstack, setCallstack] = useState<string>("");
    const fetchCallstack = async () => {
        try {
            const callstack = await invoke<string>("get_callstack", {
                damselflyInstance: activeInstance
            });
            setCallstack(callstack);
        } catch (error) {
            console.error("Failed to fetch operation log", error);
        }
    }
    useEffect(() => {
        fetchCallstack().then();
    },[xClick])

    return (
        <div className="callstack">
            {callstack}
        </div>
    )
}

export default Callstack;