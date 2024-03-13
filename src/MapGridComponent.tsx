
import {useRef, useEffect} from "react";

type Data = {
    timestamp: number;
    data: number[];
}

interface MapGridProps {
    memoryData: Data;
    blockSize: number;
}

function MapGrid({ memoryData, blockSize }: MapGridProps) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    console.log("log");
    console.log(memoryData);
    console.log(memoryData.data);

    useEffect(() => {
        if (memoryData && memoryData.data.length > 0) {
            console.log("memorydata");
            console.log(memoryData);
            drawGrid(memoryData.data, window.innerWidth);
        }
    }, [memoryData, blockSize]);


    const drawGrid = (data: number[], width: number) => {
        console.log("data");
        console.log(data);
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const blockWidth = 5;
        const gridWidth = width / 2;
        // Dynamically calculate the required height based on data length and gridWidth
        const rows = Math.ceil(data.length * blockWidth / gridWidth);
        const gridHeight = rows * blockWidth;

        // Set canvas dimensions
        canvas.width = gridWidth;
        canvas.height = gridHeight;

        ctx.clearRect(0, 0, canvas.width, canvas.height);

        let curX = -blockWidth;
        let curY = 0;

        console.log(data);
        for (let i = 0; i < data.length; ++i) {
            const curBlock = data[i];

            curX += blockWidth;
            if (curX >= canvas.width) {
                curX = 0;
                curY += blockWidth;
            }

            ctx.fillStyle = getColorForBlock(curBlock[1]);
            ctx.fillRect(curX, curY, blockWidth, blockWidth);
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


    return (
        <div>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default MapGrid;
