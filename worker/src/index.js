export default {
  async fetch(request) {
    try {
      /**
       * Gather incoming values from the request & ensure it's a curl request
       */
      const url = new URL(request.url);
      const isRootRequest = (url.pathname === "/");

      if (!isRootRequest) {
        return fetch(request);
      }

      /**
       * Response properties are immutable. To change them, construct a new
       * Response and pass modified attributes in the ResponseInit
       * object. Response headers can be modified through the headers `set` method.
       */
      const originalResponse = await fetch(request);

      let response = new Response(originalResponse.body, {
        status: originalResponse.status,
        statusText: originalResponse.statusText,
        headers: originalResponse.headers,
      });

      // Add correct MIME type for bash script
      response.headers.set("Content-Type", "text/x-shellscript");

      return response;
    } catch (err) {
      // fallback if request fails
      return fetch(request);
    }
  },
};
