// Development-only comparison with the pinned public vault grammar. Node 22+.
import { spawnSync } from 'node:child_process';
const token = '<rsv_aaaaaaaaaaaaaaaaaaaaaaaaaa>';
const markerPattern = /r\p{Cf}*s\p{Cf}*v\p{Cf}*_/giu;
const tokenPattern = /<rsv_[a-z2-7]{26}>/g;
const corpus = [token, token + token, '前' + token + '後', 'rsv_', 'RSV_', 'rſv_', '<rsv_', '<rsv_aaa>', '<rsv_' + 'A'.repeat(26) + '>', 'ordinary', 'r s v _'];
if (process.versions.unicode !== '17.0') throw new Error('Parity run requires Unicode 17.0 runtime; update table/profile deliberately.');
for (let cp = 0; cp <= 0x10ffff; cp++) {
    const c = String.fromCodePoint(cp);
    if (/\p{Cf}/u.test(c)) {
        corpus.push(`r${c}sv_`, `rs${c}v_`, `rsv${c}_`, `<rsv_${c}${'a'.repeat(26)}>`);
    }
    for (const letter of ['r', 's', 'v']) {
        if (new RegExp(`^${letter}$`, 'iu').test(c)) {
            corpus.push('rsv_'.replace(letter, c));
        }
    }
}
const run = spawnSync('target/release/examples/scan_lines', [], {
    input: corpus.map(t => Buffer.from(t).toString('hex')).join('\n') + '\n',
    encoding: 'utf8', maxBuffer: 8 * 1024 * 1024,
});
if (run.error || run.status !== 0) throw new Error('Build/run scan_lines before parity check.');
const actual = run.stdout.trim().split('\n');
if (actual.length !== corpus.length) throw new Error('Parity output count mismatch.');
for (let index = 0; index < corpus.length; index++) {
    const text = corpus[index];
    const tokens = [...text.matchAll(tokenPattern)].length;
    const markers = [...text.matchAll(markerPattern)].length;
    const expected = markers === tokens ? String(tokens) : 'err';
    if (actual[index] !== expected) throw new Error(`Grammar mismatch at synthetic case ${index}; values omitted.`);
}
console.log(`Vault grammar/Unicode parity: ${corpus.length} synthetic cases passed.`);
