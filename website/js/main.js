/**
 * ShitRust Website JavaScript
 */

document.addEventListener('DOMContentLoaded', function() {
  // Mobile menu toggle
  const hamburger = document.querySelector('.hamburger');
  const navLinks = document.querySelector('.nav-links');
  
  if (hamburger) {
    hamburger.addEventListener('click', function() {
      navLinks.classList.toggle('show');
    });
  }
  
  // Close mobile menu when clicking outside
  document.addEventListener('click', function(event) {
    if (navLinks && navLinks.classList.contains('show') && !event.target.closest('nav')) {
      navLinks.classList.remove('show');
    }
  });
  
  // Tab switching functionality
  const tabButtons = document.querySelectorAll('.tab-btn');
  const tabContents = document.querySelectorAll('.tab-content');
  
  tabButtons.forEach(button => {
    button.addEventListener('click', () => {
      // Remove active class from all buttons and content
      tabButtons.forEach(btn => btn.classList.remove('active'));
      tabContents.forEach(content => content.classList.remove('active'));
      
      // Add active class to clicked button and corresponding content
      button.classList.add('active');
      const target = button.getAttribute('data-target');
      document.getElementById(target).classList.add('active');
    });
  });
  
  // Code copy functionality
  const copyButtons = document.querySelectorAll('.copy-btn');
  
  copyButtons.forEach(button => {
    button.addEventListener('click', () => {
      const codeBlock = button.closest('.code-block').querySelector('code');
      const textToCopy = codeBlock.textContent;
      
      // Copy to clipboard
      navigator.clipboard.writeText(textToCopy)
        .then(() => {
          // Change button text temporarily
          const originalText = button.textContent;
          button.textContent = 'Copied!';
          
          // Reset button text after 2 seconds
          setTimeout(() => {
            button.textContent = originalText;
          }, 2000);
        })
        .catch(err => {
          console.error('Could not copy text: ', err);
        });
    });
  });
  
  // Smooth scrolling for anchor links
  document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function(e) {
      e.preventDefault();
      
      const targetId = this.getAttribute('href');
      const targetElement = document.querySelector(targetId);
      
      if (targetElement) {
        window.scrollTo({
          top: targetElement.offsetTop - 80, // Adjust for header height
          behavior: 'smooth'
        });
        
        // Close mobile menu if open
        if (navLinks && navLinks.classList.contains('show')) {
          navLinks.classList.remove('show');
        }
        
        // Update URL
        history.pushState(null, null, targetId);
      }
    });
  });
  
  // Highlight active nav link based on scroll position
  window.addEventListener('scroll', function() {
    const sections = document.querySelectorAll('section[id]');
    const scrollPosition = window.scrollY + 100; // Add offset for header
    
    sections.forEach(section => {
      const sectionTop = section.offsetTop;
      const sectionHeight = section.offsetHeight;
      const sectionId = section.getAttribute('id');
      
      if (scrollPosition >= sectionTop && scrollPosition < sectionTop + sectionHeight) {
        document.querySelector(`.nav-links a[href="#${sectionId}"]`)?.classList.add('active');
      } else {
        document.querySelector(`.nav-links a[href="#${sectionId}"]`)?.classList.remove('active');
      }
    });
  });
  
  // Syntax highlighting if Prism.js is available
  if (typeof Prism !== 'undefined') {
    Prism.highlightAll();
  }
}); 