import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend } from 'recharts';

type GraphDataItem = { x: number; y: number };

interface GraphProps {
    dataLoadedTrigger: boolean
}

function Graph({ dataLoadedTrigger }: GraphProps) {
  const [data, setData] = useState<GraphDataItem[]>([]); // Explicitly type the state here

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
  }, [dataLoadedTrigger]);

  return (
    <LineChart width={600} height={300} data={data}
      margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
      <CartesianGrid strokeDasharray="3 3" />
      <XAxis dataKey="x" />
      <YAxis />
      <Tooltip />
      <Legend />
      <Line type="monotone" dataKey="y" stroke="#8884d8" activeDot={{ r: 8 }} />
    </LineChart>
  );
}

export default Graph;

