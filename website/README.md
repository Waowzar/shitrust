# ShitRust Website

This directory contains the source files for the ShitRust programming language website.

## Structure

- `index.html` - Main HTML file
- `css/style.css` - Styles for the website
- `js/main.js` - JavaScript functionality
- `images/` - Contains images including the ShitRust logo
- `netlify.toml` - Configuration for Netlify deployment

## Local Development

To preview the website locally, you can use any simple HTTP server. For example:

### Using Python

```bash
# Python 3
python -m http.server

# Python 2
python -m SimpleHTTPServer
```

### Using Node.js

```bash
# Install serve globally
npm install -g serve

# Serve the website
serve .
```

Then open your browser to `http://localhost:8000` (or the port shown in your console).

## Deploying to Netlify

### Option 1: Deploy from GitHub

1. Push the entire repository to GitHub
2. Log in to [Netlify](https://www.netlify.com/)
3. Click "New site from Git" 
4. Connect to your GitHub repository
5. Set the following options:
   - Build command: (leave empty)
   - Publish directory: `website`
6. Click "Deploy site"

### Option 2: Manual Deploy

1. Install the [Netlify CLI](https://docs.netlify.com/cli/get-started/):
   ```bash
   npm install -g netlify-cli
   ```

2. Navigate to the website directory:
   ```bash
   cd website
   ```

3. Login to Netlify:
   ```bash
   netlify login
   ```

4. Deploy the site:
   ```bash
   netlify deploy --prod
   ```

## Customization

### Changing Colors

The website uses CSS variables for colors. To change the color scheme, edit the `:root` section in `css/style.css`:

```css
:root {
  --primary: #d16969;
  --primary-light: #ff9797;
  --primary-dark: #a03c3c;
  --background: #2b2b2b;
  --background-light: #3c3c3c;
  --background-dark: #1a1a1a;
  /* other variables */
}
```

### Adding Content

To add new sections or modify existing content, edit the `index.html` file.

## Notes

- The website is excluded from the main GitHub repository via `.gitignore` to keep the main repository clean.
- For deploying to GitHub Pages, you'll need to create a separate repository for the website or use a different branch. 