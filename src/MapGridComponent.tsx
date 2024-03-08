
import {useRef, useEffect, useState} from "react";
import {invoke} from "@tauri-apps/api/tauri";

type Data = {
    timestamp: number;
    data: number[];
}

interface MapGridProps {
    data: Data;
}

function MapGrid({ data }: MapGridProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const [blockSize, setBlockSize] = useState<number>(5);

    useEffect(() => {
        if (data && data[1]) {
            drawGrid(data[1], window.innerWidth);
        }
    }, [data, blockSize]);


    const drawGrid = (data: number[], width: number) => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const blockSize = 5;
        const gridWidth = width / 2;
        // Dynamically calculate the required height based on data length and gridWidth
        const rows = Math.ceil(data.length * blockSize / gridWidth);
        const gridHeight = rows * blockSize;

        // Set canvas dimensions
        canvas.width = gridWidth;
        canvas.height = gridHeight;

        ctx.clearRect(0, 0, canvas.width, canvas.height);

        let curX = -blockSize;
        let curY = 0;

        for (let i = 0; i < data.length; ++i) {
            const curBlock = data[i];

            curX += blockSize;
            if (curX >= canvas.width) {
                curX = 0;
                curY += blockSize;
            }

            ctx.fillStyle = getColorForBlock(curBlock[1]);
            ctx.fillRect(curX, curY, blockSize, blockSize);
        }
    };

    const getColorForBlock = (blockValue: number) => {
        switch(blockValue) {
            case 0: return "lightgrey";
            case 1: return "green";
            case 2: return "yellow";
            default: return "red";
        }
    };

    const handleChangeBlockSize = async (increase_by: number) => {
        setBlockSize(blockSize + increase_by);
        await invoke("set_block_size", { newBlockSize: blockSize });
        console.log(blockSize);
    };

    return (
        <div>
            <div><label>{data[0]}</label></div>
            <button onClick={() => handleChangeBlockSize(10)}>+</button>
            <button onClick={() => handleChangeBlockSize(-10)}>-</button>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default MapGrid;
