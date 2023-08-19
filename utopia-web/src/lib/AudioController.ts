import { SampleBuffer } from 'utopia-wasm-bindings';

const BUFFER_LENGTH = 8192;
const DESYNC_TOLERANCE = 0.5;

export default class AudioController {
    private ctx: AudioContext;
    private buffer: AudioBuffer;
    private bufferPos: number = 0;
    private bufferStartTime: number;

    public constructor(sampleRate: number) {
        this.ctx = new AudioContext({
            sampleRate,
        });

        this.buffer = this.ctx.createBuffer(2, BUFFER_LENGTH, this.ctx.sampleRate);
        this.bufferStartTime = this.ctx.currentTime;
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

        this.bufferStartTime += this.buffer.duration;

        let delta = this.bufferStartTime - this.ctx.currentTime;

        if (delta < 0 || delta >= DESYNC_TOLERANCE) {
            this.bufferStartTime = this.ctx.currentTime;
        }

        const bufferSource = this.ctx.createBufferSource();
        bufferSource.buffer = this.buffer;
        bufferSource.connect(this.ctx.destination);
        bufferSource.start(this.bufferStartTime);

        this.buffer = this.ctx.createBuffer(2, BUFFER_LENGTH, this.ctx.sampleRate);
        this.buffer.copyToChannel(left.slice(-this.bufferPos), 0);
        this.buffer.copyToChannel(right.slice(-this.bufferPos), 1);
    }
}
