import { ChangeEvent, useState } from 'react';
import styled from 'styled-components';

const Wrapper = styled.div`
    display: flex;
    justify-content: center;
`;

const Uploader = styled.div`
    margin: 8px 16px;
`;

export interface Rom {
    path: string;
    data: Uint8Array;
    bios: Uint8Array | null;
}

interface Props {
    onRomUpload(romFile: Rom): void;
}

export default ({ onRomUpload }: Props) => {
    const [showBiosUpload, setShowBiosUpload] = useState(false);
    const [romPath, setRomPath] = useState<string | null>(null);
    const [romData, setRomData] = useState<Uint8Array | null>(null);
    const [biosData, setBiosData] = useState<Uint8Array | null>(null);

    const onGameRomUpload = async (event: ChangeEvent<HTMLInputElement>) => {
        event.target.blur();

        const file = event.target.files?.[0];

        if (!file) {
            return;
        }

        const path = file.name;
        const data = new Uint8Array(await file.arrayBuffer());

        setRomPath(path);
        setRomData(data);

        // TODO: Can we make Utopia tell us whether it needs BIOS or not?
        const match = /\.(\w+)$/.exec(path);

        if (match?.[1] === 'sfc' || match?.[1] === 'smc') {
            // SNES, so requires IPL ROM
            setShowBiosUpload(true);

            if (!biosData) {
                return;
            }
        } else {
            setShowBiosUpload(false);
        }

        onRomUpload({
            path,
            data,
            bios: biosData,
        });
    };

    const onIplRomUpload = async (event: ChangeEvent<HTMLInputElement>) => {
        event.target.blur();

        const file = event.target.files?.[0];

        if (!file) {
            return;
        }

        const bios = new Uint8Array(await file.arrayBuffer());

        setBiosData(bios);

        if (!romPath || !romData) {
            return;
        }

        onRomUpload({
            path: romPath,
            data: romData,
            bios,
        });
    };

    return (
        <Wrapper>
            <Uploader>
                <label>
                    Upload Game ROM:{' '}
                    <input type="file" onChange={onGameRomUpload} />
                </label>
            </Uploader>
            {showBiosUpload && (
                <Uploader>
                    <label>
                        Upload IPL ROM:{' '}
                        <input type="file" onChange={onIplRomUpload} />
                    </label>
                </Uploader>
            )}
        </Wrapper>
    );
};
