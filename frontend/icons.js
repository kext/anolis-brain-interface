const icons = app => {
  const h = Vue.h
  const icon = (name, size, path) => {
    app.component('icon-' + name + '-' + size, {
      props: ['color'],
      setup(props) {
        return () => h('svg', {
          viewBox: '0 0 ' + size + ' ' + size,
          class: 'icon'
        }, [
          h('path', {
            fill: props.color || 'currentColor',
            d: path
          })
        ])
      }
    })
  }
  icon('arrow-w', '16', 'm1 8 5-5 1 1-3.25 3.25h10.25v1.5h-10.25l3.25 3.25-1 1z')
  icon('arrow-n', '16', 'm8 1 5 5-1 1-3.25-3.25v10.25h-1.5v-10.25l-3.25 3.25-1-1z')
  icon('arrow-e', '16', 'm15 8-5 5-1-1 3.25-3.25h-10.25v-1.5h10.25l-3.25-3.25 1-1z')
  icon('arrow-s', '16', 'm8 15-5-5 1-1 3.25 3.25v-10.25h1.5v10.25l3.25-3.25 1 1z')
  icon('arrow-sw', '16', 'm2.25 13.75v-6.75h1.5v4.25l8.25-8.25 1 1-8.25 8.25h4.25v1.5z')
  icon('arrow-se', '16', 'm13.75 13.75h-6.75v-1.5h4.25l-8.25-8.25 1-1 8.25 8.25v-4.25h1.5z')
  icon('arrow-nw', '16', 'm2.25 2.25h6.75v1.5h-4.25l8.25 8.25-1 1-8.25-8.25v4.25h-1.5z')
  icon('arrow-ne', '16', 'm13.75 2.25v6.75h-1.5v-4.25l-8.25 8.25-1-1 8.25-8.25h-4.25v-1.5z')
  icon('download', '16', 'm8 12-5-5 1-1 3.25 3.25v-8.25h1.5v8.25l3.25-3.25 1 1zm-5 1h10v1.5h-10z')
  icon('battery', '16', 'm3 3h2.5v-2l.5-.5h4l.5.5v2h2.5v13h-10zm4 0h2v-1h-2zm-2.5 1.5v10h7v-10zm4 1.5-.5 3h2l-2.5 4 .5-3h-2z')
  icon('repo', '16', 'm2 2.5q0-2.5 2.5-2.5h9.5v14h-4v-1.5h2.5v-2h-9v2h.5v1.5h-.5q-1.5 0-1.5-1.5zm10.5-1h-8q-1 0-1 1v6.5h9zm-7.5 10.5h4v4l-2-1.5-2 1.5z')
  icon('repo', '24', 'm21 0v22h-8v-2h6v-4h-14v4h1v2h-1q-2 0-2-2v-16q0-4 4-4zm-2 2h-12q-2 0-2 2v10h14zm-12 16h5v6l-2.5-2-2.5 2z')
  icon('book', '16', 'm0 1h5q2.25 0 3 1.5.75-1.5 3-1.5h5v12h-5q-1.5 0-2.25.75-.75.75-1.5 0-.75-.75-2.25-.75h-5zm1.5 1.5v9h3.5q1.25 0 2.25.75v-7.5q0-2.25-2.25-2.25zm13 0h-3.5q-2.25 0-2.25 2.25v7.5q1-.75 2.25-.75h3.5z')
  icon('book', '24', 'm0 3h8q3 0 4 2 1-2 4-2h8v17h-7.5q-1.5 0-3 1.5-1.5 1.5-3 0-1.5-1.5-3-1.5h-7.5zm2 2v13h5.5q2 0 3.5 1.5v-11.5q0-3-3-3zm20 0h-6q-3 0-3 3v11.5q1.5-1.5 3.5-1.5h5.5z')
  icon('calendar', '16', 'm3 0h1.5v2h7v-2h1.5v2h2v13h-14v-13h2zm-.5 3.5v2.5h11v-2.5zm0 4v6h11v-6z')
  icon('calendar', '24', 'm5 0h2v3h10v-3h2v3h3v19h-20v-19h3zm-1 5v3h16v-3zm0 5v10h16v-10z')
  icon('check', '16', 'm3 7 3 3 7-7 1 1-8 8-4-4z')
  icon('circle', '16', 'm8 0a8 8 0 018 8 8 8 0 01-8 8 8 8 0 01-8-8 8 8 0 018-8zm0 1.5a6.5 6.5 0 00-6.5 6.5 6.5 6.5 0 006.5 6.5 6.5 6.5 0 006.5-6.5 6.5 6.5 0 00-6.5-6.5z')
  icon('circle', '24', 'm12 1a11 11 0 0111 11 11 11 0 01-11 11 11 11 0 01-11-11 11 11 0 0111-11zm0 2a9 9 0 00-9 9 9 9 0 009 9 9 9 0 009-9 9 9 0 00-9-9z')
  icon('clock', '16', 'm8 0a8 8 0 018 8 8 8 0 01-8 8 8 8 0 01-8-8 8 8 0 018-8zm0 1.5a6.5 6.5 0 00-6.5 6.5 6.5 6.5 0 006.5 6.5 6.5 6.5 0 006.5-6.5 6.5 6.5 0 00-6.5-6.5zm-.75 1.5h1.5v4.25h3.25v1.5h-4.75z')
  icon('clock', '24', 'm12 1a11 11 0 0111 11 11 11 0 01-11 11 11 11 0 01-11-11 11 11 0 0111-11zm0 2a9 9 0 00-9 9 9 9 0 009 9 9 9 0 009-9 9 9 0 00-9-9zm-1 2h2v6l4 0 0 2-6 0z')
  icon('comment', '16', 'm1 1h14v11h-6l-3 3h-2v-3h-3zm1.5 1.5v8h3v2.9l2.9-2.9h5.1v-8z')
  icon('comment', '24', 'm2 2h20v16l-9.5 0-4 4-2.5 0 0-4h-4zm2 2v12h4v3.8l3.8-3.8h8.2v-12z')
  icon('database', '16', 'm1 3.5q0-3.5 7-3.5 7 0 7 3.5v8q0 3.5-7 3.5-7 0-7-3.5zm1.5 0q0 2 5.5 2 5.5 0 5.5-2 0-2-5.5-2-5.5 0-5.5 2zm0 4q0 2 5.5 2 5.5 0 5.5-2v-1.7q-1.5 1.2-5.5 1.2-4 0-5.5-1.2zm0 4q0 2 5.5 2 5.5-.1 5.5-2v-1.7q-1.5 1.2-5.5 1.2-4 0-5.5-1.2z')
  icon('database', '24', 'm2 6q0-5 10-5 10 0 10 5v12q0 5-10 5-10 0-10-5zm2 0q0 3 8 3 8 0 8-3 0-3-8-3-8 0-8 3zm0 6q0 3 8 3 8 0 8-3v-3q-2 2-8 2-6 0-8-2zm0 6q0 3 8 3 8 0 8-3v-3q-2 2-8 2-6 0-8-2z')
  icon('droplets', '16', 'm9 0q3 3 3 5 0 3-3 3-3 0-3-3 0-2 3-5zm0 2q-1.5 1.5-1.5 3 0 1.5 1.5 1.5 1.5 0 1.5-1.5 0-1.5-1.5-3zm-6 4q3 3 3 5 0 3-3 3-3 0-3-3 0-2 3-5zm0 2q-1.5 1.5-1.5 3 0 1.5 1.5 1.5 1.5 0 1.5-1.5 0-1.5-1.5-3zm10 0q3 3 3 5 0 3-3 3-3 0-3-3 0-2 3-5zm0 2q-1.5 1.5-1.5 3 0 1.5 1.5 1.5 1.5 0 1.5-1.5 0-1.5-1.5-3z')
  icon('eraser', '16', 'm11 0 5 5-7.5 7.5h5.5v1.5h-11l-3-3zm-5 7-4 4 1.5 1.5h3l2.5-2.5z')
  icon('floppy', '16', 'm1 1h11l3 3v11h-14zm1.5 1.5v11h11v-9l-2-2h-.5v4.5h-7v-4.5zm3 0v3h4v-3zm1 8a1.5 1.5 0 003 0 1.5 1.5 0 00-3 0z')
  icon('gear', '16', 'm6.1.49.6-.49 2.6 0 .6.49.41 1.91 1.38.8 1.86-.6.73.27 1.3 2.26-.12.76-1.46 1.31 0 1.6 1.46 1.31.12.76-1.3 2.26-.73.27-1.86-.6-1.38.8-.41 1.91-.6.49-2.6 0-.6-.49-.41-1.91-1.38-.8-1.86.6-.73-.27-1.3-2.26.12-.76 1.46-1.31 0-1.6-1.46-1.31-.12-.76 1.3-2.26.73-.27 1.86.6 1.38-.8zm2.5 1.01-1.2 0-.44 1.9-2.42 1.4-1.87-.57-.6 1.04 1.43 1.33 0 2.8-1.43 1.33.6 1.04 1.87-.57 2.42 1.4.44 1.9 1.2 0 .44-1.9 2.42-1.4 1.87.57.6-1.04-1.43-1.33 0-2.8 1.43-1.33-.6-1.04-1.87.57-2.42-1.4zm2.4 6.5a3 3 0 11-6 0 3 3 0 016 0zm-1.5 0a1.5 1.5 0 10-3 0 1.5 1.5 0 003 0z')
  icon('gear', '24', 'm9.66 1.48.64-.48 3.4 0 .64.48.97 3.19 1.38.8 3.25-.76.74.32 1.7 2.94-.1.8-2.28 2.43 0 1.6 2.28 2.43.1.8-1.7 2.94-.74.32-3.25-.76-1.38.8-.97 3.19-.64.48-3.4 0-.64-.48-.97-3.19-1.38-.8-3.25.76-.74-.32-1.7-2.94.1-.8 2.28-2.43 0-1.6-2.28-2.43-.1-.8 1.7-2.94.74-.32 3.25.76 1.38-.8zm3.24 1.52-1.8 0-.89 3.1-2.42 1.4-3.13-.78-.9 1.56 2.24 2.32 0 2.8-2.24 2.32.9 1.56 3.13-.78 2.42 1.4.89 3.1 1.8 0 .89-3.1 2.42-1.4 3.13.78.9-1.56-2.24-2.32 0-2.8 2.24-2.32-.9-1.56-3.13.78-2.42-1.4zm3.1 9a3 3 0 11-8 0 3 3 0 018 0zm-2 0a1.5 1.5 0 10-4 0 1.5 1.5 0 004 0z')
  icon('globe', '16', 'm8 0a8 8 0 00-8 8 8 8 0 008 8 8 8 0 008-8 8 8 0 00-8-8zm0 1.5a6.5 6.5 0 011.64.21l.12.13.15.03.18.13-.22-.02.01.1.09.14-.3-.21-.24.07.11.06 0 .12.13.2-.13.07-.13.01.33.29-.34-.04-.31-.11-.02-.23.08-.09-.06-.04-.14.14.05.28.29.04.13.09-.24.03-.18 0 .01.09-.15-.02.03.17-.19.22-.29-.24.28.06.13-.08-.37-.14-.35-.12-.47.19-.35.34-.28.12-.07.25.07.08.15-.05.12-.01.04.26.04.08.19-.1.07-.16.14-.11-.08-.09.02-.12.2-.16.03-.1.1-.04.1.07-.23.16 0 .2.08.06.31-.05-.02.07-.26.07-.01.16-.11-.04-.05.22-.13.08-.06-.07-.18.09-.16-.06-.17-.01-.01-.1.06-.08.03-.11-.14.06-.05.12-.02.17-.06-.02-.23.09-.31.2-.32.1-.15 0-.03.06.12.06.07.15-.16.28-.42-.11-.11.06-.19.39-.05.25.17-.03.04.16.11-.03.24-.01.22-.2.14-.19.22-.12.09-.13.18.08.27-.09.09.17.34.25.04.16-.09.07-.25.04.23.11.08-.13.01.05.13-.14-.01-.15.11.08.04-.05-.24-.17-.22-.27-.02-.11.1-.05.07.07.4.37-.04.12.11.22.19.22.17-.19-.14-.21.11-.06.19.04.23-.06-.12-.21.29-.38.18.04-.08.05.09.13.21-.1-.13-.11.3-.12-.11.26.45.25-.2.15-.45-.12-.37.15-.28.1.05.29.24.15.62-.05-.11.54-.47.08-.79-.22-.15.07-.08.21-.88-.43.12-.1 0-.26-.64-.07-.53.17-.26-.1-.18.15-.25.14-.08.27-.39.16-.5.56.09.31-.3.58.31.74.48.54.63.07.63-.08.15.24.3-.05.19.18-.14.45.37.57.15.64-.17.5.34.49.21.5.33.5.67-.04.32-.12.37-.33.05-.21.3-.15-.05-.38.66-.46.03-.46-.1-.1.02-.69.76-.73.41-.71 0-.4-.7.28-.16-.2-.41-.38-.38-.69-.34-.71.16.05.57.77.5.92 1.07-.72.35-.64-.33-.13-.1-.24-.15.29-.35-.14-.33-.41.16-.1.17.22.37.12.12-.1.2.17.66-.17.51.46.06-.24.12.45.53.99.15-.36-.08-.57.11-.39.22-.7.31.46.39.53.1 0a6.5 6.5 0 01.02.57 6.5 6.5 0 01-6.5 6.5 6.5 6.5 0 01-6.5-6.5 6.5 6.5 0 011.5-4.15l.1.05.3-.1.7-.4.9-1-.29 0a6.5 6.5 0 013.29-.9zm-1.3.4-.5.3-1.1.6-.1.3.8-.3.6-.1 1-.6-.7-.2zm2.3.11.08.07.29.03-.01-.08-.36-.02zm-.44.17-.07.04.06.04.1-.04-.09-.04zm-.51.08-.31.06.08.06.02.11.09-.09.13.04-.12-.13.1.04.01-.09zm-1.94.67-.11.02.05.05-.12.03.05.15.35-.05.01-.15-.25.02.02-.07zm.29.77-.2.1 0 .3-.3.4.2-.1.4-.1-.1-.3.1-.2-.1-.1zm-.5.3-.2.2.1.1.2-.1.1-.2-.2 0zm4.25.81.11.1 0 .25.3.21.11.45-.4-.06-.42-.79.3-.16zm3.31 3.39.06.41.13-.14-.19-.27zm-2.66 3.05-.56.53-.02.28-.17.2.05.32.27-.07.54-.9-.11-.36z')
  icon('info', '24', 'm12 1a11 11 0 0111 11 11 11 0 01-11 11 11 11 0 01-11-11 11 11 0 0111-11zm0 2a9 9 0 00-9 9 9 9 0 009 9 9 9 0 009-9 9 9 0 00-9-9zm-1 3h2v2h-2zm-2 4h4v6h2v2h-6v-2h2v-4h-2z')
  icon('markdown', '16', 'm15 3q1 0 1 1v8q0 1-1 1h-14q-1 0-1-1v-8q0-1 1-1zm-6 8v-6h-2l-1.5 2-1.5-2h-2v6h2v-3l1.5 2 1.5-2v3zm3 .5 2.5-3.5h-1.5v-3h-2v3h-1.5z')
  icon('minus', '16', 'm2 7.25h12v1.5h-12z')
  icon('pencil', '16', 'm11.7.7h1l2.6 2.6v1l-9.6 9.6-3.9 1.1-.8-.8 1-3.8zm2 3-1.4-1.4-1.4 1.4 1.4 1.4zm-2.5 2.5-1.4-1.4-6.4 6.4-.5 1.9 1.9-.5z')
  icon('plus', '16', 'm2 7.25h5.25v-5.25h1.5v5.25h5.25v1.5h-5.25v5.25h-1.5v-5.25h-5.25z')
  icon('pulse', '16', 'm5.5 2h1l3.5 8.7 1.5-3.7h4.5v1.5h-3.5l-2 5h-1l-3.5-8.8-1.5 3.8h-4.5v-1.5h3.49z')
  icon('pulse', '24', 'm0 11h5l3-8h2l5 15 2.6-7h6.4v2h-5l-3 8h-2l-5-15-2.6 7h-6.4z')
  icon('rocket', '16', 'm14.5 0q1.5 0 1.5 1.5 0 5.5-4 8v4.1l-3.7 2.4-1-.4-1-3.2-2.7-2.7-3.2-1-.4-1 2.4-3.7h4.1q2.5-4 8-4zm-9.5 8.9 2.1 2.1 3.9-2.7q3.5-2.3 3.5-6.8-4.5 0-6.8 3.3zm-1.3 5.8-2.7 1.3h-1v-1l1.3-2.7 1.7-.3 1 1zm6.8-4.2-2.6 1.8.5 1.8 2.1-1.3zm-6.8-2.4 1.8-2.6h-2.2l-1.4 2.1zm8.3-3.1a1 1 90 11-2 0 1 1 90 012 0z')
  icon('server', '16', 'm0 1.5.5-.5h15l.5.5v5.75l-.5.5.5.5v5.75l-.5.5h-15l-.5-.5v-5.75l.5-.5-.5-.5zm1.5 1v4.5h13v-4.5zm0 6v4.5h13v-4.5zm1.5-4.5h2v1.5h-2zm4 0h6v1.5h-6zm-4 6h2v1.5h-2zm4 0h6v1.5h-6z')
  icon('server', '24', 'm1 2.5.5-.5h21l.5.5v9l-.5.5.5.5v9l-.5.5h-21l-.5-.5v-9l.5-.5-.5-.5zm2 1.5v7h18v-7zm0 9v7h18v-7zm3-6h2v1.5h-2zm4 0h8v1.5h-8zm-4 9h2v1.5h-2zm4 0h8v1.5h-8z')
  icon('spinner', '16', 'm8 1.5a6.5 6.5 0 00-6.5 6.5 6.5 6.5 0 006.5 6.5 6.5 6.5 0 006.5-6.5h1.5a8 8 0 01-8 8 8 8 0 01-8-8 8 8 0 018-8z')
  icon('square', '16', 'm1 1h14v14h-14zm1.5 1.5v11h11v-11z')
  icon('swap', '16', 'm2 4h11.25l-1.5-1.5 1-1 3.25 3.25-3.25 3.25-1-1 1.5-1.5h-11.25zm12 6.5v1.5h-11.25l1.5 1.5-1 1-3.25-3.25 3.25-3.25 1 1-1.5 1.5z')
  icon('sync', '16', 'm2.5 8a5.5 5.5 0 005.5 5.5 5.5 5.5 0 004.13-1.87l-1.63-1.63h4.5v4.5l-1.8-1.8a7 7 0 01-5.2 2.3 7 7 0 01-7-7zm11 0a5.5 5.5 0 00-5.5-5.5 5.5 5.5 0 00-4.13 1.87l1.63 1.63h-4.5v-4.5l1.8 1.8a7 7 0 015.2-2.3 7 7 0 017 7z')
  icon('thermometer', '16', 'm6 2q0-2 2-2 2 0 2 2v6.53a4 4 0 012 3.47 4 4 0 01-4 4 4 4 0 01-4-4 4 4 0 012-3.47zm2.5 0q0-.5-.5-.5-.5 0-.5.5v7.55a2.5 2.5 0 00-2 2.45 2.5 2.5 0 003 2.5h0a2.5 2.5 0 002-2.5 2.5 2.5 0 00-2-2.45zm2.5-1h2v1h-2zm0 3h2v1h-2zm0 3h2v1h-2z')
  icon('trash', '16', 'm2 3h3v-2.5l.5-.5h5l.5.5v2.5h3v1.5h-12zm4.5 0h3v-1.5h-3zm-3.5 3h1.5l1 7.5h5l1-7.5h1.5l-1 9h-8z')
  icon('wifi', '24', 'm0 10a12 12 0 013.51-8.49l1.42 1.42a10 10 0 00-2.93 7.07 10 10 0 002.93 7.07l-1.42 1.42a12 12 0 01-3.51-8.49zm5 0a7 7 0 012.05-4.95l1.41 1.41a5 5 0 00-1.46 3.54 5 5 0 001.46 3.54l-1.41 1.41a7 7 0 01-2.05-4.95zm5 0a2 2 0 012-2 2 2 0 012 2 2 2 0 01-1 1.73v9.27h-2v-9.27a2 2 0 01-1-1.73zm7 0a5 5 0 00-1.46-3.54l1.41-1.46a7 7 0 012.05 5 7 7 0 01-2.05 4.95l-1.41-1.41a5 5 0 001.46-3.54zm5 0a10 10 0 00-2.93-7.07l1.42-1.42a12 12 0 013.51 8.49 12 12 0 01-3.51 8.49l-1.42-1.42a10 10 0 002.93-7.07z')
  icon('x', '16', 'm3 4 1-1 4 4 4-4 1 1-4 4 4 4-1 1-4-4-4 4-1-1 4-4z')
  icon('zap', '16', 'm9 2-1 5.25h4l-5 6.75 1-5.25h-4z')
}
