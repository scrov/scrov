// /platform: phone menu and copy button. The page works without this file.
(() => {
  const menuBtn = document.querySelector('.menu-btn');
  const links = document.getElementById('nav-links');

  if (menuBtn && links) {
    const setOpen = (open) => {
      menuBtn.setAttribute('aria-expanded', String(open));
      links.classList.toggle('open', open);
    };
    menuBtn.addEventListener('click', () => setOpen(menuBtn.getAttribute('aria-expanded') !== 'true'));
    links.addEventListener('click', (event) => {
      if (event.target.closest('a')) setOpen(false);
    });
    document.addEventListener('keydown', (event) => {
      if (event.key === 'Escape' && menuBtn.getAttribute('aria-expanded') === 'true') {
        setOpen(false);
        menuBtn.focus();
      }
    });
    window.matchMedia('(min-width: 881px)').addEventListener('change', (mq) => {
      if (mq.matches) setOpen(false);
    });
  }

  for (const button of document.querySelectorAll('[data-copy]')) {
    button.addEventListener('click', async () => {
      const source = document.getElementById(button.dataset.copy);
      const status = document.getElementById(`${button.dataset.copy}-status`);
      if (!source) return;
      try {
        await navigator.clipboard.writeText(source.textContent.trim());
        if (status) status.textContent = 'Copied.';
      } catch {
        if (status) status.textContent = 'Copy failed. Select the command and copy it.';
      }
      if (status) setTimeout(() => { status.textContent = ''; }, 2500);
    });
  }
})();
