// Live search: any input with data-search-url fetches an HTML fragment from
// the server and swaps it into the element matched by data-search-target.
for (const input of document.querySelectorAll("[data-search-url]")) {
  const target = document.querySelector(input.dataset.searchTarget);
  let timer;

  input.addEventListener("input", () => {
    clearTimeout(timer);

    // Wait until the user stops typing for 300ms.
    timer = setTimeout(async () => {
      const url = `${input.dataset.searchUrl}?q=${encodeURIComponent(input.value)}`;
      const response = await fetch(url);
      target.innerHTML = await response.text();
    }, 300);
  });
}
