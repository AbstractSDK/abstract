mermaid.initialize({
  startOnLoad:true,
  theme: ['light', 'rust', 'abstract-light'].includes(window.localStorage.getItem('mdbook-theme')) ? 'default' : 'dark',
});
