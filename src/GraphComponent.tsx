
import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine } from 'recharts';

type GraphDataItem = { x: number; y: number };

interface GraphProps {
    dataLoaded: boolean,
    setXClick: (x: number) => void;
    xClick: number;
    setXLimit: (x: number) => void;
}

function Graph({ dataLoaded , setXClick , xClick, setXLimit }: GraphProps) {
    const [usageData, setUsageData] = useState<GraphDataItem[]>([]);
    const [fragmentationData, setFragmentationData] = useState<GraphDataItem[]>([]);
    const [chartWidth, setChartWidth] = useState(window.innerWidth / 2);
    const [chartHeight, _setChartHeight] = useState(300); // Maintain a fixed height or adjust as needed

    const updateDimensions = useCallback(() => {
        setChartWidth(window.innerWidth / 2); // Set width to half of window width
        // Optionally, adjust height here if needed
    }, []);

    const fetchData = async () => {
        try {
            const usageData: Array<[number, number]> = await invoke('get_viewer_usage_graph');
            const formattedUsageData = usageData.map((item): GraphDataItem => ({ x: item[0], y: item[1] }));
            let formattedData;

            
            const fragmentationData: Array<[number, number]> = await invoke('get_viewer_fragmentation_graph');
            const formattedFragmentationData = fragmentationData.map((item): GraphDataItem => ({ x: item[0], y: item[1] }));
            console.log(formattedUsageData);
            console.log(formattedFragmentationData);
            setUsageData(formattedUsageData);
            setFragmentationData(formattedFragmentationData);
            setXLimit(usageData.length - 1);
        } catch (error) {
            console.error('Error fetching graph data:', error);
        }
    };

    useEffect(() => {
        fetchData().then();
    }, [dataLoaded]);

    useEffect(() => {
        window.addEventListener('resize', updateDimensions);
        return () => window.removeEventListener('resize', updateDimensions);
    }, [updateDimensions]);

    const handlePointClick = (e: any) => {
        if (e && e.activePayload && e.activePayload.length > 0) {
            const xValue = e.activePayload[0].payload.x;
            setXClick(xValue);
        }
    };

    return (
        <LineChart width={chartWidth} height={chartHeight}
                   onClick={handlePointClick}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="x" />
            <YAxis />
            <Tooltip />
            <Legend />
            <Line data={usageData} type="monotone" dataKey="y" stroke="#8884d8" dot={false} activeDot={false} />
            <Line data={fragmentationData} type="monotone" dataKey="y" stroke="#82ca9d" dot={false} activeDot={false} />
            {dataLoaded && <ReferenceLine x={xClick} stroke="red" label="Selected" />}
        </LineChart>
    );
}

export default Graph;
