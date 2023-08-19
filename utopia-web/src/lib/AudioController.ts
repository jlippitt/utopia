import { SampleBuffer } from 'utopia-wasm-bindings';

export default class AudioController {
    private ctx: AudioContext;

    public constructor(sampleRate: number) {
        this.ctx = new AudioContext({
            sampleRate,
        });

        this.ctx.resume();
    }

    public close() {
        this.ctx.close();
    }

    public send(sampleBuffer: SampleBuffer) {
        const left = sampleBuffer.getLeft();
        const right = sampleBuffer.getRight();

        const audioBuffer = this.ctx.createBuffer(
            2,
            left.length + right.length,
            this.ctx.sampleRate
        );

        audioBuffer.copyToChannel(left, 0);
        audioBuffer.copyToChannel(right, 1);

        const bufferSource = this.ctx.createBufferSource();
        bufferSource.buffer = audioBuffer;
        bufferSource.connect(this.ctx.destination);
        bufferSource.start(0);
    }
}
