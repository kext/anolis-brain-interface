const plot = app => {
  const scale = (v, min, max) => 1.8 * (v - min) / (max - min) - 0.9
  const genData = (data, width, height, enlarge = true) => {
    let waveform = new Float32Array(data.length * 4)
    let zeroline = new Float32Array(8)
    let min = data[0][0]
    let max = data[0][1]
    for (let i = 1; i < data.length; ++i) {
      if (min > data[i][0]) min = data[i][0]
      if (max < data[i][1]) max = data[i][1]
    }
    if (min >= max) {
      min -= 1
      max += 1
    }
    let n = data.length - 1
    for (let i = 0; i <= n; ++i) {
      let a = scale(data[i][0], min, max)
      let b = scale(data[i][1], min, max)
      let h = enlarge ? Math.max((2 / height - (b - a)), 0) : 0
      waveform[i * 4 + 0] = 2 * i / n - 1
      waveform[i * 4 + 1] = b + h
      waveform[i * 4 + 2] = 2 * i / n - 1
      waveform[i * 4 + 3] = a - h
    }
    const zero = scale(0, min, max)
    for (let i = 0; i <= 1; ++i) {
      let h = 1 / height
      zeroline[i * 4 + 0] = 2 * i - 1
      zeroline[i * 4 + 1] = zero + h
      zeroline[i * 4 + 2] = 2 * i - 1
      zeroline[i * 4 + 3] = zero - h
    }
    return {
      waveform,
      zeroline,
      zero,
      min,
      max
    }
  }
  const svgns = 'http://www.w3.org/2000/svg'
  app.component('plot-2d', {
    props: {
      data: {
        default: []
      },
      label: {
        type: String,
        default: ''
      },
      color: {
        default: { r: 3 / 15, g: 6 / 15, b: 10 / 15 }
      }
    },
    data() {
      return {
        stop: false
      }
    },
    setup() {
      const div = Vue.h('div', {class: 'plot'})
      return () => div
    },
    beforeUnmount() {
      this.stop = true
    },
    mounted() {
      const div = this.$el
      console.log(div)
      const label = document.createElement('div')
      const maxText = document.createElement('div')
      const minText = document.createElement('div')
      label.appendChild(document.createTextNode(''))
      label.classList.add('label')
      maxText.appendChild(document.createTextNode(' '))
      maxText.classList.add('max')
      minText.appendChild(document.createTextNode(' '))
      minText.classList.add('min')
      const svg = document.createElementNS(svgns, 'svg')
      const zerolinePath = document.createElementNS(svgns, 'path')
      const waveformPath = document.createElementNS(svgns, 'path')
      let w = 500
      let h = 300
      let oldViewBox = '0 0 ' + w + ' ' + h
      let oldWaveform = 'M0 0'
      let oldZeroline = 'M0 0'
      let oldColor = this.color
      zerolinePath.setAttribute('fill', 'none')
      zerolinePath.setAttribute('stroke', '#999')
      zerolinePath.setAttribute('stroke-width', '2px')
      zerolinePath.setAttribute('d', oldZeroline)
      waveformPath.setAttribute('fill', oldColor)
      waveformPath.setAttribute('stroke', oldColor)
      waveformPath.setAttribute('stroke-width', '2px')
      waveformPath.setAttribute('stroke-linejoin', 'round')
      waveformPath.setAttribute('d', oldWaveform)
      const update = () => {
        if (this.stop) {
          return
        }
        const r = div.getBoundingClientRect()
        if (w !== r.width || h !== r.height) {
          w = r.width
          h = r.height
        }
        const viewBox = '0 0 ' + w + ' ' + h
        if (viewBox !== oldViewBox) {
          oldViewBox = viewBox
          svg.setAttribute('viewBox', viewBox)
          //svg.style.width = w + 'px'
          //svg.style.height = h + 'px'
          svg.setAttribute('width', w + 'px')
          svg.setAttribute('height', h + 'px')
        }
        const x = v => ((1 + v) * 0.5 * w).toFixed(2)
        const y = v => ((1 - v) * 0.5 * h).toFixed(2)
        const color = this.color
        let zeroline = 'M0 0'
        let waveform = 'M0 0'
        if (this.data.length > 1) {
          let d = genData(this.data, w, h, false)
          zeroline = 'M0 ' + y(d.zero) + ' ' + x(1) + ' ' + y(d.zero)
          waveform = 'M'
          for (let i = 0; i < d.waveform.length; i += 4) {
            waveform += x(d.waveform[i]) + ' ' + y(d.waveform[i + 1]) + ' '
          }
          for (let i = d.waveform.length - 4; i >= 0; i -= 4) {
            waveform += x(d.waveform[i + 2]) + ' ' + y(d.waveform[i + 3]) + ' '
          }
          waveform += 'Z'
          minText.firstChild.nodeValue = d.min.toFixed(3)
          maxText.firstChild.nodeValue = d.max.toFixed(3)
        } else {
          minText.firstChild.nodeValue = ' '
          maxText.firstChild.nodeValue = ' '
        }
        if (oldZeroline !== zeroline) {
          oldZeroline = zeroline
          zerolinePath.setAttribute('d', zeroline)
        }
        if (oldWaveform !== waveform) {
          oldWaveform = waveform
          waveformPath.setAttribute('d', waveform)
        }
        if (oldColor !== color) {
          oldColor = color
          waveformPath.setAttribute('fill', color)
          waveformPath.setAttribute('stroke', color)
        }
        label.firstChild.nodeValue = this.label
        requestAnimationFrame(update)
      }
      update()
      svg.appendChild(zerolinePath)
      svg.appendChild(waveformPath)
      div.appendChild(svg)
      div.appendChild(label)
      div.appendChild(maxText)
      div.appendChild(minText)
    }
  })
}
