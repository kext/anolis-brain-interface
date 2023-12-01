const fs = require('fs')
const zlib = require('zlib')

const loadFile = name => {
  const data = zlib.gunzipSync(fs.readFileSync(name))
  if (new TextDecoder().decode(data.subarray(0, 8)) !== '<SALEAE>') throw new Error('Invalid signature')
  if (data.readInt32LE(8) != 0) throw new Error('Wrong version')
  if (data.readInt32LE(12) != 1) throw new Error('Expected analogue trace')
  const samples = Number(data.readBigUInt64LE(40))
  const result = new Float32Array(samples)
  for (let i = 0; i < samples; ++i) {
    result[i] = data.readFloatLE(i * 4 + 48)
  }
  return {
    samples: result,
    rate: Number(data.readBigUInt64LE(24))
  }
}

const reduction = 50
const len = 0.15
const offset = 0.04
const file = loadFile('2500sps-8ch-zerodbm-2m.bin.gz')
let tsv = 'time\tcurrent\n'
const n = Math.floor(file.rate / reduction * len) + 1
const o = Math.round(offset * file.rate)
console.log(n)
for (let i = 0; i < n; ++i) {
  let v = 0
  for (let j = 0; j < reduction; ++j) {
    v += file.samples[i * reduction + j + o]
  }
  const time = i * (1000 / file.rate * reduction)
  tsv += `${time.toFixed(2)}\t${(v / reduction / 68 * 1000).toFixed(1)}\n`
}
fs.writeFileSync('trace-2500sps-8ch-zerodbm-2m.dat', tsv)
