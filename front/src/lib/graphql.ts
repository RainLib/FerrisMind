const GRAPHQL_ENDPOINT = "http://localhost:4000/graphql";

export async function fetchGraphQL<T>(
  query: string,
  variables: Record<string, unknown> = {},
): Promise<{ data?: T; errors?: Array<{ message: string }> }> {
  try {
    const response = await fetch(GRAPHQL_ENDPOINT, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        query,
        variables,
      }),
    });

    const result = await response.json();
    return result;
  } catch (error) {
    console.error("GraphQL Fetch Error:", error);
    return { errors: [{ message: String(error) }] };
  }
}

// Queries
export const GET_NOTEBOOKS = `
  query GetNotebooks {
    notebooks {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`;

// Mutations
export const CREATE_NOTEBOOK = `
  mutation CreateNotebook($name: String!, $description: String) {
    createNotebook(input: { name: $name, description: $description }) {
      id
      name
    }
  }
`;

export const UPDATE_NOTEBOOK = `
  mutation UpdateNotebook($id: String!, $name: String, $description: String) {
    updateNotebook(input: { id: $id, name: $name, description: $description }) {
      id
      name
      description
    }
  }
`;

export const DELETE_NOTEBOOK = `
  mutation DeleteNotebook($id: String!) {
    deleteNotebook(id: $id)
  }
`;

export type Notebook = {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
};
