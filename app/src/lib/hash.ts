/// Hex SHA-256 of a UTF-8 string, matching the server's blob hashing. Used to
/// tell whether a file's live text still equals its recorded blob — i.e. its
/// "saved" state — for the editor tab's unsaved-changes dot. Web Crypto needs a
/// secure context (dev is localhost, prod is https), same as the rest of the app.
export async function sha256Hex(text: string): Promise<string> {
  const bytes = new TextEncoder().encode(text);
  const digest = await crypto.subtle.digest('SHA-256', bytes);
  return [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('');
}
