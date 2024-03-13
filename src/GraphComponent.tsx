import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import {LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ReferenceLine} from 'recharts';

type GraphDataItem = { x: number; y: number };

interface GraphProps {
    dataLoaded: boolean,
    setXClick: (x: number) => void;
    xClick: number;
}

function Graph({ dataLoaded , setXClick , xClick }: GraphProps) {
  const [data, setData] = useState<GraphDataItem[]>([]);

  const fetchData = async () => {
    try {
      const graphData: Array<[number, number]> = await invoke('get_viewer_graph');
      const formattedData = graphData.map((item): GraphDataItem => ({ x: item[0], y: item[1] }));
      setData(formattedData);
    } catch (error) {
      console.error('Error fetching graph data:', error);
    }
  };

  useEffect(() => {
    fetchData();
  }, [dataLoaded]);

  const handlePointClick = (e: any) => {
    if (e && e.activePayload && e.activePayload.length > 0) {
      const xValue = e.activePayload[0].payload.x;
      setXClick(xValue);
    }
  };

  return (
    <LineChart width={600} height={300} data={data}
      onClick={handlePointClick}>
      <CartesianGrid strokeDasharray="3 3" />
      <XAxis dataKey="x" />
      <YAxis />
      <Tooltip />
      <Legend />
      <Line type="monotone" dataKey="y" stroke="#8884d8" activeDot={{ r: 8 }} />
        {dataLoaded && <ReferenceLine x={xClick} stroke="red" label="Selected" />}
    </LineChart>
  );
}

export default Graph;

