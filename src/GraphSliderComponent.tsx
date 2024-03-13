import { styled } from '@mui/material/styles';
import Box from '@mui/material/Box';
import Grid from '@mui/material/Grid';
import Typography from '@mui/material/Typography';
import Slider from '@mui/material/Slider';
import MuiInput from '@mui/material/Input';

interface SliderProps {
    xClick: number;
    setXClick: (x: number) => void;
    xLimit: number;
}

const Input = styled(MuiInput)`
    width: 42px;
`;

function GraphSlider({ xClick, setXClick, xLimit }: SliderProps) {
    const handleSliderChange = (_event: Event, newValue: number | number[]) => {
        setXClick(Math.min(newValue as number, xLimit));
    };

    const handleInputChange = (event: React.ChangeEvent<HTMLInputElement>) => {
        setXClick(event.target.value === '' ? 0 : Number(event.target.value));
    };

    const handleBlur = () => {
        if (xClick < 0) {
            setXClick(0);
        }
    }

    console.log(xLimit);

    return (
        <Box className="slider" sx={{ width: 250 }}>
            <Typography id="input-slider" gutterBottom>
                Time
            </Typography>
            <Grid item xs>
                <Slider
                    value={typeof xClick === "number" ? xClick : 0}
                    onChange={handleSliderChange}
                    aria-labelledby="input-slider"
                    min={0}
                    max={xLimit}
                />
            </Grid>
            <Grid item>
                <Input
                    value={xClick}
                    size="small"
                    onChange={handleInputChange}
                    onBlur={handleBlur}
                    inputProps={{
                        step: 1,
                        min: 0,
                        max: {xLimit},
                        type: 'number',
                        'aria-labelledby': 'input-slider',
                    }}
                />
            </Grid>
        </Box>
    );
}

export default GraphSlider;
