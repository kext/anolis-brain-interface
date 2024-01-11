//! Convert a recorded file to CSV.
//! Usage: node data2csv.js [input] [output]

const fs = require('fs')

const readData = file => {
  let data = fs.readFileSync(file)
  let pos = 0
  let result = []
  while (pos < data.length) {
    if (pos + 16 > data.length) {
      throw new Error('Malformed file')
    }
    if (data.readUInt32LE(pos) !== 0x55daba) {
      throw new Error('Malformed file')
    }
    let len = data.readUint32LE(pos + 4)
    if (len < 16) {
      throw new Error('Malformed file')
    }
    let ts = Number(data.readBigInt64LE(pos + 8))
    result.push({
      ts, data: data.subarray(pos + 16, pos + len)
    })
    pos += len
  }
  return result
}

const main = () => {
  let file = process.argv[2]
  let out = process.argv[3]
  if (!file) {
    console.error('Missing input file name')
    process.exit(1)
  }
  if (!out) {
    console.error('Missing output file name')
    process.exit(1)
  }
  let data = readData(file)
  console.log(data.length)
  let csv = 'T,C1,C2,C3,C4,C5,C6,C7,C8\n'
  let T = 0
  data.forEach(packet => {
    if (packet.data[1] !== 8) return
    let pos = 2
    let frame = []
    while (pos < packet.data.length) {
      frame.push(packet.data.readUInt16LE(pos))
      if (frame.length === 8) {
        csv += T + ',' + frame.join(',') + '\n'
        frame.length = 0
        T += 1
      }
      pos += 2
    }
  })
  fs.writeFileSync(out, csv)
}
main()
