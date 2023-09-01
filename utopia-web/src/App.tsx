import { useRef } from 'react';
import FileUpload, { Rom } from './components/FileUpload';
import styled from 'styled-components';
import * as utopia from 'utopia-wasm-bindings';

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

export default () => {
    const canvasRef = useRef<HTMLCanvasElement>(null);

    const onRomUpload = (rom: Rom) => {
        if (!canvasRef.current) {
            return;
        }

        utopia.run(
            canvasRef.current,
            rom.path,
            rom.data,
            rom.bios ?? undefined
        );
    };

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <CanvasWrapper>
                <div>
                    <canvas ref={canvasRef} />
                </div>
            </CanvasWrapper>
        </Wrapper>
    );
};
