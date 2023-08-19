import { useEffect, useRef, useState } from 'react';
import Canvas from './components/Canvas';
import FileUpload from './components/FileUpload';
import styled from 'styled-components';
import { Utopia } from 'utopia-wasm-bindings';

const DEFAULT_WIDTH = 256;
const DEFAULT_HEIGHT = 224;
const DEFAULT_PIXELS = (() => {
    const pixels = new Uint8ClampedArray(DEFAULT_WIDTH * DEFAULT_HEIGHT * 4);

    let index = 0;

    while (index < pixels.length) {
        pixels[index++] = 0;
        pixels[index++] = 0;
        pixels[index++] = 0;
        pixels[index++] = 0xff;
    }

    return pixels;
})();

export const Wrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
`;

export default () => {
    const frameRef = useRef(0);
    const utopiaRef = useRef<Utopia | null>(null);

    const [width, setWidth] = useState(DEFAULT_WIDTH);
    const [height, setHeight] = useState(DEFAULT_HEIGHT);
    const [pixels, setPixels] = useState(DEFAULT_PIXELS);

    const runFrame = (_timestamp: number) => {
        const utopia = utopiaRef.current;

        if (utopia) {
            utopia.runFrame();
            setWidth(utopia.getScreenWidth());
            setHeight(utopia.getScreenHeight());
            setPixels(utopia.getPixels());
        }

        frameRef.current = requestAnimationFrame(runFrame);
    };

    const onRomUpload = async (file: File) => {
        utopiaRef.current?.free();
        const data = new Uint8Array(await file.arrayBuffer());
        utopiaRef.current = new Utopia(file.name, data);
    };

    useEffect(() => {
        frameRef.current = requestAnimationFrame(runFrame);
        return () => cancelAnimationFrame(frameRef.current);
    }, []);

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <Canvas width={width} height={height} pixels={pixels} />
        </Wrapper>
    );
};
