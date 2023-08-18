import styled from 'styled-components';

export const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
`;

export default () => (
    <Wrapper>
        <div>Canvas goes here</div>
    </Wrapper>
);
