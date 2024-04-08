import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
interface BlockStatusProps {
    selectedBlock: number;
    timestamp: number;
}

function BlockStatus({ selectedBlock, timestamp }: BlockStatusProps) {
    const get_block_updates = async (selectedBlock: number, timestamp: number) => {
        return await invoke("query_block", {
            address: selectedBlock,
            timestamp: timestamp
        });
    }

    useEffect(() => {
        const block_updates = get_block_updates(selectedBlock, timestamp);
        console.log(block_updates);
    }, [selectedBlock]);

    return (
        <div className="blockstatus">
            test
        </div>
    )
}

export default BlockStatus;