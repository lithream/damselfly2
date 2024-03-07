import {useRef, useEffect} from "react";

type BlockStatus = "Allocated" | "PartiallyAllocated" | "Free" | "Unused";

interface MapGridProps {
    data: BlockStatus[];
}

function MapGrid({ data }: MapGridProps ) {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    useEffect(() => {
        if (data) {
            console.log("redrawing data");
            console.log(data);
            drawGrid(data);
        }
    }, [data]);

    const drawGrid = (data: string[]) => {
        const canvas = canvasRef.current;
        if (!canvas) return;
        const ctx = canvas.getContext("2d");
        if (!ctx) return;

        const blockSize = 5;
        const gridWidth = Math.sqrt(data.length);

        ctx.clearRect(0, 0, canvas.width, canvas.height);

        canvas.width = gridWidth * blockSize;
        canvas.height = gridWidth * blockSize;

        let blocksTillTruncate = 256;
        let prevBlock = data[0];
        let curBlock;
        let truncations = 0;

        for (let i = 0; i < data.length; ++i) {
            curBlock = data[i];

            if (curBlock === prevBlock) {
                blocksTillTruncate--;
                if (blocksTillTruncate <= 0) {
                    truncations++;
                    continue;
                } 
            } else {
                blocksTillTruncate = 256; // Reset counter if block state changes
                prevBlock = curBlock;
            }

            let parts = curBlock.split(' ');

            let relativeI = i - truncations;

            const x = (relativeI % gridWidth) * blockSize; // Fixed calculation
            const y = Math.floor(relativeI / gridWidth) * blockSize;

            switch(parts[0]) {
                case "A":
                    ctx.fillStyle = "red";
                    break;
                case "P":
                    ctx.fillStyle = "yellow";
                    break;
                case "F":
                    ctx.fillStyle = "green";
                    break;
                default:
                    ctx.fillStyle = "lightgrey";
            }

            ctx.fillRect(x, y, blockSize, blockSize);
        }
    };

    return (
        <div>
            <canvas ref={canvasRef} />
        </div>
    );
}

export default MapGrid;