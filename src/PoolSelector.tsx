
import React from 'react';

interface PoolSelectorProps {
    poolList: {name: string, index: number}[];
    selectedPool: number;
    onSelectPool: (index: number) => void;
}

const PoolSelector: React.FC<PoolSelectorProps> = ({ poolList, selectedPool, onSelectPool }) => {
    return (
        <div>
            <label htmlFor="poolSelector">Select Memory Pool:</label>
            <select id="poolSelector" value={selectedPool} onChange={(e) => onSelectPool(parseInt(e.target.value))}>
                {poolList.map((pool, index) => (
                    <option key={index} value={pool.index}>
                        {pool.name}
                    </option>
                ))}
            </select>
        </div>
    );
};

export default PoolSelector;
