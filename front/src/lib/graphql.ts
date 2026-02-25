const GRAPHQL_ENDPOINT = "http://localhost:8080/graphql";

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

export const GET_NOTEBOOK = `
  query GetNotebook($id: String!) {
    notebook(id: $id) {
      id
      name
      description
      createdAt
      updatedAt
    }
  }
`;

export const GET_NOTEBOOK_INITIAL_DATA = `
  query GetNotebookInitialData($notebookId: String!) {
    notebook(id: $notebookId) {
      id
      name
      description
      createdAt
      updatedAt
    }
    documents(notebookId: $notebookId) {
      id
      filename
      uploadStatus
      chunkCount
      sha256
      createdAt
      updatedAt
      summary
    }
    notebookChatHistory(notebookId: $notebookId, limit: 50, offset: 0) {
      sessionId
      messages {
        id
        role
        content
        createdAt
      }
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

export const GET_NOTEBOOK_DOCUMENTS = `
  query GetNotebookDocuments($notebookId: String!) {
    documents(notebookId: $notebookId) {
      id
      filename
      uploadStatus
      chunkCount
      sha256
      createdAt
      updatedAt
      summary
    }
  }
`;

export const GET_DOCUMENT_UPLOAD_STATUSES = `
  query DocumentUploadStatuses($ids: [String!]!) {
    documentUploadStatuses(ids: $ids) {
      id
      filename
      uploadStatus
      chunkCount
      sha256
    }
  }
`;

export const GET_DOCUMENT_CONTENT = `
  query DocumentContent($documentId: String!) {
    documentContent(documentId: $documentId) {
      documentId
      filename
      uploadStatus
      summary
      chunks {
        index
        content
      }
      images {
        imageId
        mimeType
        sourceRef
        storedPath
      }
    }
  }
`;

export const SUMMARIZE_DOCUMENT = `
  mutation SummarizeDocument($documentId: String!) {
    summarizeDocument(documentId: $documentId) {
      documentId
      summary
    }
  }
`;

export const DELETE_DOCUMENT = `
  mutation DeleteDocument($id: String!) {
    deleteDocument(id: $id)
  }
`;

export type Notebook = {
  id: string;
  name: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
};

export type Document = {
  id: string;
  filename: string;
  uploadStatus: string;
  chunkCount: number;
  sha256: string;
  createdAt: string;
  updatedAt: string;
  summary: string | null;
};

export type DocumentUploadStatus = {
  id: string;
  filename: string;
  uploadStatus: string;
  chunkCount: number;
  sha256: string;
};

export type DocumentContent = {
  documentId: string;
  filename: string;
  uploadStatus: string;
  summary: string | null;
  chunks: { index: number; content: string }[];
  images: {
    imageId: string;
    mimeType: string;
    sourceRef: string;
    storedPath: string;
  }[];
};

export type ChatHistoryMessage = {
  id: string;
  role: string;
  content: string;
  createdAt: string;
};

export type ChatHistoryPage = {
  sessionId: string;
  messages: ChatHistoryMessage[];
};

export type NotebookInitialData = {
  notebook: Notebook | null;
  documents: Document[];
  notebookChatHistory: ChatHistoryPage;
};
