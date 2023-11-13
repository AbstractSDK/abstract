mermaid.initialize({
  startOnLoad:true,
  theme: ['light', 'rust'].includes(window.localStorage.getItem('mdbook-theme')) ? 'default' : 'dark',
});
