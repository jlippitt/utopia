import { useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: top;
    height: 100%;
`;

interface CanvasProps {
    $scaleFactor: number;
}

const Canvas = styled.canvas<CanvasProps>`
    image-rendering: optimizeSpeed;
    image-rendering: crisp-edges;
    image-rendering: -moz-crisp-edges;
    image-rendering: -o-crisp-edges;
    image-rendering: -webkit-optimize-contrast;
    -ms-interpolation-mode: nearest-neighbor;

    ${(props) => `
        width: ${props.$scaleFactor * +(props.width ?? 0)}px;
        height: ${props.$scaleFactor * +(props.height ?? 0)}px;
    `}
`;

interface Props {
    width: number;
    height: number;
    pixels: Uint8ClampedArray;
}

export default ({ width, height, pixels }: Props) => {
    const wrapperRef = useRef<HTMLDivElement>(null);
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const ctxRef = useRef<CanvasRenderingContext2D | null>(null);
    const imageRef = useRef<ImageData | null>(null);

    const [scaleFactor, setScaleFactor] = useState(1);

    useEffect(() => {
        if (!canvasRef.current) {
            return;
        }

        const ctx = canvasRef.current.getContext('2d', {
            alpha: false,
            willReadFrequently: true,
        });

        if (!ctx) {
            return;
        }

        ctxRef.current = ctx;
        imageRef.current = ctx.getImageData(0, 0, width, height);
        ctx.imageSmoothingEnabled = false;

        if (!wrapperRef.current) {
            return;
        }

        const bounds = wrapperRef.current.getBoundingClientRect();
        const maxWidthScale = Math.floor(bounds.width / width);
        const maxHeightScale = Math.floor(bounds.height / height);
        setScaleFactor(Math.min(maxWidthScale, maxHeightScale));
    }, [width, height]);

    useEffect(() => {
        if (!ctxRef.current || !imageRef.current) {
            return;
        }

        imageRef.current?.data.set(pixels);
        ctxRef.current.putImageData(imageRef.current, 0, 0);
    }, [pixels]);

    return (
        <Wrapper ref={wrapperRef}>
            <Canvas
                ref={canvasRef}
                width={width}
                height={height}
                $scaleFactor={scaleFactor}
            />
        </Wrapper>
    );
};
