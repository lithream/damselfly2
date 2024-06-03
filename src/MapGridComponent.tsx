import { useState, useRef, useEffect } from "react";
import Data from "./Data.tsx";

interface MapGridProps {
    memoryData: Data;
    blockSize: number;
    squareSize: number;
    selectedBlock: number;
    setSelectedBlock: (block: number) => void;
}

function MapGrid({ memoryData, blockSize, squareSize, selectedBlock, setSelectedBlock }: MapGridProps) {
    const [selectedIndexDebug, setSelectedIndexDebug] = useState<number>(0);
    const canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        if (memoryData && memoryData.data.length > 0) {
            drawGrid(memoryData.data, window.innerWidth);
        }

        // event listener for clicks
        const canvas = canvasRef.current;
        if (canvas) {
            canvas.addEventListener('click', handleCanvasClick);
        }

        return () => {
            if (canvas) {
                canvas.removeEventListener('click', handleCanvasClick);
            }
        }
    }, [selectedBlock, squareSize, memoryData, blockSize]);

    const handleCanvasClick = (event: MouseEvent) => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const rect = canvas.getBoundingClientRect();
        const x = event.clientX - rect.left;
        const y = event.clientY - rect.top;

        const col = Math.floor(x / squareSize);
        const row = Math.floor(y / squareSize);

        const index = row * (Math.round(canvas.width / squareSize)) + col;
        console.log(`Block clicked at row: ${row}, col: ${col}, index: ${index}`);
        console.log(`Setting selected block to: 0x${memoryData.data[index][0].toString(16)}`);
        setSelectedBlock(memoryData.data[index][0]);
        setSelectedIndexDebug(index);
    }


    const drawGrid = (data: number[][], width: number) => {
        console.log(selectedBlock);
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const blockWidth = squareSize;
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

        for (let i = 0; i < data.length; ++i) {
            const curBlock = data[i];

            curX += blockWidth;
            if (curX >= canvas.width) {
                curX = 0;
                curY += blockWidth;
            }

            if (curBlock[0] == selectedBlock && curBlock[1] != -1) {
                ctx.fillStyle = "green";
            } else {
                ctx.fillStyle = getColorForBlock(curBlock[1]);
            }
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
            <div>Selected Block Index: {selectedBlock}, {selectedIndexDebug}</div>  {/* Debug Window to show the selected block index */}
        </div>
    );
}

export default MapGrid;
