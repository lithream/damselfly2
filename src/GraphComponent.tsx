
import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine } from 'recharts';

interface GraphProps {
    activeInstance: number,
    dataLoaded: boolean,
    realtimeGraph: boolean,
    setXClick: (x: number) => void;
    xClick: number;
    setXLimit: (x: number) => void;
    setRealtimeGraphOffset: (x: number) => void;
}

type GraphData = {
    timestamp: number,
    usage: number,
    fragmentation: number,
    largest_free_block: number,
    free_blocks: number,
}

function Graph({ activeInstance, dataLoaded, realtimeGraph, setXClick , xClick, setXLimit, setRealtimeGraphOffset }: GraphProps) {
    const [data, setData] = useState<GraphData[]>([]);
    const [chartWidth, setChartWidth] = useState(window.innerWidth / 2);
    const [chartHeight, _setChartHeight] = useState(300); // Maintain a fixed height or adjust as needed

    const updateDimensions = useCallback(() => {
        setChartWidth(window.innerWidth / 2); // Set width to half of window width
    }, []);

    const trim_blank_start_from_graphs = (
        usageData: Array<[number, number]>,
        fragmentationData: Array<[number, number]>,
        largestFreeBlockData: Array<[number, number]>,
        freeBlocksData: Array<[number, number]>) => {

        let trimmedUsageData: Array<[number, number]> = [];
        let trimmedFragmentationData: Array<[number, number]> = [];
        let trimmedLargestFreeBlockData: Array<[number, number]> = [];
        let trimmedFreeBlocksData: Array<[number, number]> = [];
        let realtimeGraphOffset: number = 0;

        let in_blank_area = true;
        for (let i = 0; i < usageData.length; i++) {
            if (in_blank_area
                && (usageData[i][1] != 0
                    || fragmentationData[i][1] != 0
                    || largestFreeBlockData[i][1] != 0
                    || freeBlocksData[i][1] != 0)
                ) {
                in_blank_area = false;
            }
            if (!in_blank_area) {
                trimmedUsageData.push(usageData[i]);
                trimmedFragmentationData.push(fragmentationData[i]);
                trimmedLargestFreeBlockData.push(largestFreeBlockData[i]);
                trimmedFreeBlocksData.push(freeBlocksData[i]);
            } else {
                realtimeGraphOffset += 1;
            }
        }

        setRealtimeGraphOffset(realtimeGraphOffset);
        return [trimmedUsageData, trimmedFragmentationData, trimmedLargestFreeBlockData, trimmedFreeBlocksData];
    }

    const fetchData = async () => {
        try {
            let usageData: Array<[number, number]>;
            let fragmentationData: Array<[number, number]>;
            let largestFreeBlockData: Array<[number, number]>;
            let freeBlocksData: Array<[number, number]>;

            if (realtimeGraph) {
                usageData = await invoke('get_viewer_usage_graph_sampled', { damselflyInstance: activeInstance });
                fragmentationData = await invoke('get_viewer_distinct_blocks_graph_sampled', { damselflyInstance: activeInstance });
                largestFreeBlockData = await invoke('get_viewer_largest_block_graph_sampled', { damselflyInstance: activeInstance });
                freeBlocksData = await invoke('get_viewer_free_blocks_graph_sampled', { damselflyInstance: activeInstance });
                let trimmedData = trim_blank_start_from_graphs(usageData, fragmentationData, largestFreeBlockData, freeBlocksData);
                usageData = trimmedData[0];
                fragmentationData = trimmedData[1];
                largestFreeBlockData = trimmedData[2];
                freeBlocksData = trimmedData[3];
            } else {
                usageData = await invoke('get_viewer_usage_graph', { damselflyInstance: activeInstance  });
                fragmentationData = await invoke('get_viewer_distinct_blocks_graph', { damselflyInstance: activeInstance });
                largestFreeBlockData = await invoke('get_viewer_largest_block_graph', { damselflyInstance: activeInstance });
                freeBlocksData = await invoke('get_viewer_free_blocks_graph', { damselflyInstance: activeInstance });
            }

            let formattedData = [];
            for (let i = 0; i < usageData.length; i++) {
                let usage = usageData[i][1];
                let fragmentation = fragmentationData[i][1];
                let largestFreeBlock = largestFreeBlockData[i][1];
                let freeBlocks = freeBlocksData[i][1];
                let datapoint: GraphData = {
                    timestamp: i,
                    usage: usage,
                    fragmentation: fragmentation,
                    largest_free_block: largestFreeBlock,
                    free_blocks: freeBlocks,
                };
                formattedData.push(datapoint);
            }

            setData(formattedData);
            setXLimit(usageData.length - 1);
        } catch (error) {
            console.error('Error fetching graph data:', error);
        }
    };

    useEffect(() => {
        fetchData().then();
    }, [dataLoaded, realtimeGraph, activeInstance]);

    useEffect(() => {
        window.addEventListener('resize', updateDimensions);
        return () => window.removeEventListener('resize', updateDimensions);
    }, [updateDimensions]);

    const handlePointClick = (e: any) => {
        console.log("Graph clicked");
        console.log(e);
        if (e) {
            setXClick(Math.round(e.activeTooltipIndex));
        }
    };

    return (
        <LineChart width={chartWidth} height={chartHeight} data={data}
                   onClick={handlePointClick}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="x" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line type="monotone" dataKey="usage" stroke="#8884d8" dot={false} activeDot={false} />
            <Line type="monotone" dataKey="fragmentation" stroke="#82ca9d" dot={false} activeDot={false} />
            <Line type="monotone" dataKey="free_blocks" stroke="#82ffff" dot={false} activeDot={false} />
            {dataLoaded && <ReferenceLine x={xClick} stroke="red" label="Selected" />}
        </LineChart>
    );
}

export default Graph;
