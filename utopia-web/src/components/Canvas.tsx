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
    let ctxRef = useRef<CanvasRenderingContext2D | null>(null);
    let imageRef = useRef<ImageData | null>(null);

    useEffect(() => {
        if (!canvasRef.current) {
            return;
        }

        ctxRef.current = canvasRef.current.getContext('2d', {
            alpha: false,
            willReadFrequently: true,
        });

        if (!ctxRef.current) {
            return;
        }

        imageRef.current = ctxRef.current.getImageData(0, 0, width, height);
    }, [width, height]);

    useEffect(() => {
        if (!ctxRef.current || !imageRef.current) {
            return;
        }

        imageRef.current?.data.set(pixels);
        ctxRef.current.putImageData(imageRef.current, 0, 0);
    }, [pixels]);

    return (
        <Wrapper>
            <canvas ref={canvasRef} width={width} height={height} />
        </Wrapper>
    );
};
