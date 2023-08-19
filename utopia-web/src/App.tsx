import { useEffect, useRef, useState } from 'react';
import Canvas from './components/Canvas';
import FileUpload, { Rom } from './components/FileUpload';
import styled from 'styled-components';
import { Utopia, JoypadState } from 'utopia-wasm-bindings';

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

const KEY_MAP = new Map<string, number>([
    ['KeyZ', 0],
    ['KeyX', 1],
    ['KeyA', 2],
    ['KeyS', 3],
    ['KeyD', 4],
    ['KeyC', 5],
    ['Space', 8],
    ['Enter', 9],
    ['ArrowUp', 12],
    ['ArrowDown', 13],
    ['ArrowLeft', 14],
    ['ArrowRight', 15],
]);

export const Wrapper = styled.div`
    display: flex;
    flex-direction: column;
    height: 100%;
    background-color: #111;
    color: #eee;
    font-family: monospace;
`;

export default () => {
    const frameRef = useRef(0);
    const utopiaRef = useRef<Utopia | null>(null);
    const keyStateRef = useRef(Array(17).fill(false));
    const [width, setWidth] = useState(DEFAULT_WIDTH);
    const [height, setHeight] = useState(DEFAULT_HEIGHT);
    const [pixels, setPixels] = useState(DEFAULT_PIXELS);

    const runFrame = () => {
        const utopia = utopiaRef.current;

        if (utopia) {
            const joypadState = new JoypadState();
            const gamepad = navigator.getGamepads()[0];

            if (gamepad) {
                for (let index = 0; index < gamepad.axes.length; ++index) {
                    joypadState.setAxis(index, gamepad.axes[index]);
                }

                for (let index = 0; index < gamepad.buttons.length; ++index) {
                    joypadState.setButton(
                        index,
                        gamepad.buttons[index].pressed
                    );
                }
            } else {
                const keyState = keyStateRef.current;

                for (let index = 0; index < keyState.length; ++index) {
                    joypadState.setButton(index, keyState[index]);
                }
            }

            utopia.runFrame(joypadState);

            setWidth(utopia.getScreenWidth());
            setHeight(utopia.getScreenHeight());
            setPixels(utopia.getPixels());
        }

        frameRef.current = requestAnimationFrame(runFrame);
    };

    const onRomUpload = async (rom: Rom) => {
        utopiaRef.current?.free();

        utopiaRef.current = new Utopia(
            rom.path,
            rom.data,
            rom.bios ?? undefined
        );
    };

    const onKeyEvent = (value: boolean) => (event: KeyboardEvent) => {
        const index = KEY_MAP.get(event.code);

        if (index !== undefined) {
            keyStateRef.current[index] = value;
        }
    };

    useEffect(() => {
        const onKeyDown = onKeyEvent(true);
        const onKeyUp = onKeyEvent(false);

        window.addEventListener('keydown', onKeyDown);
        window.addEventListener('keyup', onKeyUp);
        frameRef.current = requestAnimationFrame(runFrame);

        return () => {
            window.removeEventListener('keydown', onKeyUp);
            window.removeEventListener('keyup', onKeyUp);
            cancelAnimationFrame(frameRef.current);
        };
    }, []);

    return (
        <Wrapper>
            <FileUpload onRomUpload={onRomUpload} />
            <Canvas width={width} height={height} pixels={pixels} />
        </Wrapper>
    );
};
