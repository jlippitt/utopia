import { useEffect, useRef } from 'react';
import styled from 'styled-components';

export const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
`;

interface Props {
    width: number;
    height: number;
}

export default ({ width, height }: Props) => {
    let canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        if (width === 0 || height === 0) {
            return;
        }

        let canvas = canvasRef.current;

        if (!canvas) {
            return;
        }

        let ctx = canvas.getContext('2d');

        if (!ctx) {
            return;
        }

        const image = ctx.createImageData(width, height);

        let index = 0;

        while (index < image.data.length) {
            image.data[index++] = 0;
            image.data[index++] = 0;
            image.data[index++] = 0;
            image.data[index++] = 0xff;
        }

        ctx.putImageData(image, 0, 0);
    });

    return (
        <Wrapper>
            <canvas ref={canvasRef} width={width} height={height} />
        </Wrapper>
    );
};
