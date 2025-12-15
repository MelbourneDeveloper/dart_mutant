// dart_mutant Website JavaScript

// Animate elements on scroll
document.addEventListener('DOMContentLoaded', () => {
  // Intersection Observer for fade-in animations
  const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px',
  };

  const observer = new IntersectionObserver((entries) => {
    entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('animate-in');
        observer.unobserve(entry.target);
      }
    });
  }, observerOptions);

  // Observe all animatable elements
  document.querySelectorAll('.feature-card, .step, .stat-item').forEach((el) => {
    el.style.opacity = '0';
    observer.observe(el);
  });

  // Animate score bar on scroll
  const scoreDemo = document.querySelector('.score-demo');
  if (scoreDemo) {
    const scoreObserver = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const fill = entry.target.querySelector('.score-fill');
            if (fill) {
              setTimeout(() => {
                fill.style.width = fill.dataset.score || '87%';
              }, 200);
            }
            scoreObserver.unobserve(entry.target);
          }
        });
      },
      { threshold: 0.5 }
    );

    scoreObserver.observe(scoreDemo);
  }

  // Copy code blocks on click
  document.querySelectorAll('pre').forEach((pre) => {
    pre.addEventListener('click', async () => {
      const code = pre.querySelector('code');
      if (code) {
        try {
          await navigator.clipboard.writeText(code.textContent);

          // Show copied feedback
          const feedback = document.createElement('div');
          feedback.textContent = 'Copied!';
          feedback.style.cssText = `
            position: absolute;
            top: 8px;
            right: 8px;
            background: var(--color-primary);
            color: var(--bg-darkest);
            padding: 4px 8px;
            border-radius: 4px;
            font-size: 12px;
            font-weight: 600;
          `;
          pre.style.position = 'relative';
          pre.appendChild(feedback);

          setTimeout(() => feedback.remove(), 1500);
        } catch (err) {
          console.log('Copy failed:', err);
        }
      }
    });
  });

  // Smooth scroll for anchor links
  document.querySelectorAll('a[href^="#"]').forEach((anchor) => {
    anchor.addEventListener('click', (e) => {
      e.preventDefault();
      const target = document.querySelector(anchor.getAttribute('href'));
      if (target) {
        target.scrollIntoView({ behavior: 'smooth' });
      }
    });
  });

  // Mobile nav toggle
  const navToggle = document.querySelector('.nav-toggle');
  const navLinks = document.querySelector('.nav-links');

  if (navToggle && navLinks) {
    // Close nav when clicking outside
    document.addEventListener('click', (e) => {
      if (!navToggle.contains(e.target) && !navLinks.contains(e.target)) {
        navLinks.classList.remove('active');
      }
    });
  }
});

// Typing animation for hero code block
function typeWriter(element, text, speed = 50) {
  let i = 0;
  element.textContent = '';

  function type() {
    if (i < text.length) {
      element.textContent += text.charAt(i);
      i++;
      setTimeout(type, speed);
    }
  }

  type();
}
