import Canvas from './components/Canvas';
import FileUpload from './components/FileUpload';
import styled from 'styled-components';
import * as utopia from 'utopia-wasm-bindings';

utopia.greet();

export const Wrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
`;

export default () => (
    <Wrapper>
        <FileUpload />
        <Canvas />
    </Wrapper>
);
