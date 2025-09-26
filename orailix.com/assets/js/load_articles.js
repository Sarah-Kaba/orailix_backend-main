function fetchArticles(categories = 'all') {
    const apiUrl = categories === 'all'
        ? '/api/articles'
        : `/api/articles?categories=${categories.join(',')}`;

    fetch(apiUrl)
        .then(response => response.json())
        .then(articles => {
            const container = document.querySelector('[role="list"].blog-grid');
            container.innerHTML = ''; // Clear existing items

            articles.forEach(article => {
                const articleHTML = `
                    <div role="listitem" class="blur-sibling-item w-dyn-item">
                        <a href="${article.link}" class="text-decoration-none w-inline-block">
                            <div class="image-wrapper border-radius-16px mg-bottom-24px">
                                <img src="${article.picture_url}" 
                                     alt="${article.title}"
                                     sizes="(max-width: 479px) 92vw, (max-width: 767px) 94vw, (max-width: 991px) 46vw, (max-width: 1439px) 29vw, 380px"
                                     class="image">
                            </div>
                            <div class="flex-horizontal---justify-start mg-bottom-16px wrap-8px">
                                <div class="text-200 text-uppercase color-neutral-600">${article.category}</div>
                                <div class="divider-details bg-neutral-900"></div>
                                <div class="text-200 text-uppercase color-neutral-600">${article.formatted_date}</div>
                            </div>
                            <h3 class="heading-h3-size mg-bottom-0 title">${article.title}</h3>
                        </a>
                    </div>
                `;
                container.insertAdjacentHTML('beforeend', articleHTML);
            });
        })
        .catch(error => console.error('Error loading articles:', error));
}

// Initial load of all articles
fetchArticles(['tech', 'presentations', 'publications']);