import { SampleBuffer } from 'utopia-wasm-bindings';

const BUFFER_LENGTH = 8192;

export default class AudioController {
    private ctx: AudioContext;
    private buffer: AudioBuffer;
    private bufferPos: number = 0;

    public constructor(sampleRate: number) {
        this.ctx = new AudioContext({
            sampleRate,
        });

        this.buffer = this.ctx.createBuffer(2, BUFFER_LENGTH, this.ctx.sampleRate);

        this.ctx.resume();
    }

    public close() {
        this.ctx.close();
    }

    public send(sampleBuffer: SampleBuffer) {
        const left = sampleBuffer.getLeft();
        const right = sampleBuffer.getRight();

        this.buffer.copyToChannel(left, 0, this.bufferPos);
        this.buffer.copyToChannel(right, 1, this.bufferPos);
        this.bufferPos += left.length;

        if (this.bufferPos < this.buffer.length) {
            return;
        }

        this.bufferPos -= this.buffer.length;

        const bufferSource = this.ctx.createBufferSource();
        bufferSource.buffer = this.buffer;
        bufferSource.connect(this.ctx.destination);
        bufferSource.start(0);

        this.buffer = this.ctx.createBuffer(2, BUFFER_LENGTH, this.ctx.sampleRate);
        this.buffer.copyToChannel(left.slice(-this.bufferPos), 0);
        this.buffer.copyToChannel(right.slice(-this.bufferPos), 1);
    }
}
