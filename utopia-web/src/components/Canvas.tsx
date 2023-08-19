import { useEffect, useRef, useState } from 'react';
import styled from 'styled-components';

const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: top;
    height: 100%;
`;

interface Props {
    width: number;
    height: number;
    pixels: Uint8ClampedArray;
}

export default ({ width, height, pixels }: Props) => {
    const wrapperRef = useRef<HTMLDivElement>(null);
    const targetCanvasRef = useRef<HTMLCanvasElement>(null);
    const targetCtxRef = useRef<CanvasRenderingContext2D | null>(null);
    const sourceCanvasRef = useRef<OffscreenCanvas | null>(null);
    const sourceCtxRef = useRef<OffscreenCanvasRenderingContext2D | null>(null);
    const imageRef = useRef<ImageData | null>(null);

    const [scaleFactor, setScaleFactor] = useState(1);

    useEffect(() => {
        if (!targetCanvasRef.current) {
            return;
        }

        sourceCanvasRef.current = new OffscreenCanvas(width, height);

        const sourceCtx = sourceCanvasRef.current.getContext('2d', {
            alpha: false,
            willReadFrequently: true,
        });

        const targetCtx = targetCanvasRef.current.getContext('2d', {
            alpha: false,
            willReadFrequently: true,
        });

        if (!sourceCtx || !targetCtx) {
            return;
        }

        sourceCtx.imageSmoothingEnabled = false;
        targetCtx.imageSmoothingEnabled = false;

        sourceCtxRef.current = sourceCtx;
        targetCtxRef.current = targetCtx;

        imageRef.current = sourceCtx.getImageData(0, 0, width, height);

        if (!wrapperRef.current) {
            return;
        }

        const bounds = wrapperRef.current.getBoundingClientRect();
        const maxWidthScale = Math.floor(bounds.width / width);
        const maxHeightScale = Math.floor(bounds.height / height);
        setScaleFactor(Math.min(maxWidthScale, maxHeightScale));
    }, [width, height]);

    useEffect(() => {
        if (
            !sourceCanvasRef.current ||
            !sourceCtxRef.current ||
            !targetCtxRef.current ||
            !imageRef.current
        ) {
            return;
        }

        imageRef.current?.data.set(pixels);

        sourceCtxRef.current.putImageData(imageRef.current, 0, 0);

        targetCtxRef.current.drawImage(
            sourceCanvasRef.current,
            0,
            0,
            width * scaleFactor,
            height * scaleFactor
        );
    }, [pixels]);

    return (
        <Wrapper ref={wrapperRef}>
            <canvas
                ref={targetCanvasRef}
                width={width * scaleFactor}
                height={height * scaleFactor}
            />
        </Wrapper>
    );
};
