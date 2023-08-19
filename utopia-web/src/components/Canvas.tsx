import { useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

export const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: top;
`;

interface Props {
    width: number;
    height: number;
    pixels: Uint8ClampedArray;
}

export default ({ width, height, pixels }: Props) => {
    let canvasRef = useRef<HTMLCanvasElement>(null);

    useEffect(() => {
        let canvas = canvasRef.current;

        if (!canvas) {
            return;
        }

        let ctx = canvas.getContext('2d');

        if (!ctx) {
            return;
        }

        let image = ctx.getImageData(0, 0, width, height);
        image.data.set(pixels);
        ctx.putImageData(image, 0, 0);
    }, [width, height, pixels]);

    return (
        <Wrapper>
            <canvas ref={canvasRef} width={width} height={height} />
        </Wrapper>
    );
};
