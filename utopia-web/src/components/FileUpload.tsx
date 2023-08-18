import styled from 'styled-components';

export const Wrapper = styled.div`
    display: flex;
    justify-content: center;
`;

export default () => (
    <Wrapper>
        <div>
            Upload ROM: <input type="file" />
        </div>
    </Wrapper>
);
