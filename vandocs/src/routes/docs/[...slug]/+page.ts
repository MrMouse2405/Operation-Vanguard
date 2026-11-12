import { error } from '@sveltejs/kit';
import { emails } from '$lib/mail';

export function entries() { return emails.map(email => ({ slug: email.slug })); }
export function load({ params }: { params: { slug: string } }) {
  const email = emails.find(email => email.slug === params.slug.replace(/\/$/, ''));
  if (!email) error(404, 'Email not found');
  return { slug: email.slug };
}
