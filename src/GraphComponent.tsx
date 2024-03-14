
import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine } from 'recharts';

interface GraphProps {
    dataLoaded: boolean,
    setXClick: (x: number) => void;
    xClick: number;
    setXLimit: (x: number) => void;
}

type GraphData = {
    timestamp: number,
    usage: number,
    fragmentation: number,
}

function Graph({ dataLoaded , setXClick , xClick, setXLimit }: GraphProps) {
    const [data, setData] = useState<GraphData[]>([]);
    const [chartWidth, setChartWidth] = useState(window.innerWidth / 2);
    const [chartHeight, _setChartHeight] = useState(300); // Maintain a fixed height or adjust as needed

    const updateDimensions = useCallback(() => {
        setChartWidth(window.innerWidth / 2); // Set width to half of window width
        // Optionally, adjust height here if needed
    }, []);

    const fetchData = async () => {
        try {
            const usageData: Array<[number, number]> = await invoke('get_viewer_usage_graph');
            const fragmentationData: Array<[number, number]> = await invoke('get_viewer_fragmentation_graph');

            let formattedData = [];
            for (let i = 0; i < usageData.length; i++) {
                let usage = usageData[i][1];
                let fragmentation = fragmentationData[i][1];
                let datapoint: GraphData = {
                    timestamp: i,
                    usage: usage,
                    fragmentation: fragmentation,
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
    }, [dataLoaded]);

    useEffect(() => {
        window.addEventListener('resize', updateDimensions);
        return () => window.removeEventListener('resize', updateDimensions);
    }, [updateDimensions]);

    const handlePointClick = (e: any) => {
        if (e) {
            setXClick(Math.round(e.activeCoordinate.x));
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
            {dataLoaded && <ReferenceLine x={xClick} stroke="red" label="Selected" />}
        </LineChart>
    );
}

export default Graph;
