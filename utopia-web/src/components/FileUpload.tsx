import { ChangeEvent } from 'react';
import styled from 'styled-components';

export const Wrapper = styled.div`
    display: flex;
    justify-content: center;
    padding: 8px;
`;

interface Props {
    onRomUpload(romFile: File): void;
}

export default ({ onRomUpload }: Props) => {
    const onFileUploadChange = async (event: ChangeEvent<HTMLInputElement>) => {
        const romFile = event.target.files?.[0];

        if (!romFile) {
            return;
        }

        onRomUpload(romFile);
    };

    return (
        <Wrapper>
            <div>
                Upload ROM: <input type="file" onChange={onFileUploadChange} />
            </div>
        </Wrapper>
    );
};
