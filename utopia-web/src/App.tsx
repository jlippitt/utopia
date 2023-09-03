import { useEffect, useRef } from 'react';
import FileUpload, { Rom } from './components/FileUpload';
import styled from 'styled-components';
import { Utopia } from 'utopia-wasm-bindings';

export const Wrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #111;
    color: #eee;
    font-family: monospace;
`;

const CanvasWrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
`;

const Canvas = styled.canvas`
    outline: none;
    -webkit-tap-highlight-color: rgba(255, 255, 255, 0);
`;

export default () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);
    const canvasWrapperRef = useRef<HTMLDivElement>(null);
    const utopiaRef = useRef<Utopia | null>(null);

    const onRomUpload = (rom: Rom) => {
        const canvas = canvasRef.current;

        if (!canvas) {
            return;
        }

        const utopia = (utopiaRef.current ||= new Utopia());

        utopia.reset(canvas, rom.path, rom.data, rom.bios ?? undefined);
    };

    useEffect(() => {
        const canvasWrapper = canvasWrapperRef.current;

        if (!canvasWrapper) {
            return;
        }

        const resizeObserver = new ResizeObserver(() => {
            const utopia = utopiaRef.current;
            utopia && utopia.updateViewport();
        });

        resizeObserver.observe(canvasWrapper);

        return () => resizeObserver.disconnect();
    });

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <CanvasWrapper ref={canvasWrapperRef}>
                <Canvas ref={canvasRef} />
            </CanvasWrapper>
        </Wrapper>
    );
};
