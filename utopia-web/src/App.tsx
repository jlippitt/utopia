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
    const onRomUpload = async (file: File) => {
        const data = new Uint8Array(await file.arrayBuffer());
        const utopia = new Utopia(file.name, data);
        console.log(utopia.getScreenWidth());
        console.log(utopia.getScreenHeight());
    };

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <Canvas />
        </Wrapper>
    );
};
