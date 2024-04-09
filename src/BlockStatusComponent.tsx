
import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";

interface BlockStatusProps {
    selectedBlock: number;
    timestamp: number;
}

interface MemoryUpdate {
    address: number;
    size: number;
    callstack: string;
    timestamp: number;
    real_timestamp: string;
}

interface Allocation extends MemoryUpdate {}
interface Free extends MemoryUpdate {}

// A wrapper type that could be returned from the backend
type MemoryUpdateType = {
    Allocation?: Allocation;
    Free?: Free;
};

function BlockStatus({ selectedBlock, timestamp }: BlockStatusProps) {
    const [memoryUpdates, setMemoryUpdates] = useState<MemoryUpdateType[]>([]);

    useEffect(() => {
        const fetchBlockUpdates = async () => {
            try {
                const updates: MemoryUpdateType[] = await invoke("query_block", {
                    address: selectedBlock,
                    timestamp: timestamp
                });
                console.log(`updates length ${updates.length}`);
                setMemoryUpdates(updates);
            } catch (error) {
                console.error("Error fetching block updates:", error);
            }
        };

        fetchBlockUpdates();
    }, [selectedBlock, timestamp]);

    const renderUpdate = (update: MemoryUpdateType) => {
        // Determine if it's an Allocation or Free
        const isAllocation = update.hasOwnProperty('Allocation');
        const updateData = isAllocation ? update.Allocation : update.Free;

        return (
            <div style={{ padding: '10px', borderBottom: '1px solid #ccc' }}>
                <div><strong>Type:</strong> {isAllocation ? "Allocation" : "Free"}</div>
                <div><strong>Block Address:</strong> {selectedBlock}</div>
                <div><strong>Operation Address:</strong> {updateData?.address}</div>
                <div><strong>Size:</strong> {updateData?.size}</div>
                <div><strong>Timestamp:</strong> {updateData?.timestamp} ({updateData?.real_timestamp})</div>
                <div><strong>Callstack:</strong> <pre>{updateData?.callstack}</pre></div>
            </div>
        );
    };

    return (
        <div className="blockstatus" style={{ overflowY: 'scroll', maxHeight: '400px' }}>
            {memoryUpdates.map((update, index) => (
                <div key={index}>
                    {renderUpdate(update)}
                </div>
            ))}
        </div>
    );
}

export default BlockStatus;
