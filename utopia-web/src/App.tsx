import { useState } from 'react';
import Canvas from './components/Canvas';
import FileUpload from './components/FileUpload';
import styled from 'styled-components';
import { Utopia } from 'utopia-wasm-bindings';

export const Wrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
`;

export default () => {
    const [screenWidth, setScreenWidth] = useState(0);
    const [screenHeight, setScreenHeight] = useState(0);

    const onRomUpload = async (file: File) => {
        const data = new Uint8Array(await file.arrayBuffer());
        const utopia = new Utopia(file.name, data);
        setScreenWidth(utopia.getScreenWidth());
        setScreenHeight(utopia.getScreenHeight());
        utopia.free();
    };

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <Canvas width={screenWidth} height={screenHeight} />
        </Wrapper>
    );
};
