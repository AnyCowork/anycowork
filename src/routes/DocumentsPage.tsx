import { useNavigate } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { PlusCircle } from "lucide-react";
import { useCreateDocument } from "@/hooks/useDocuments";
import { toast } from "sonner";

const DocumentsPage = () => {
  const navigate = useNavigate();
  const createDocument = useCreateDocument();

  const onCreate = () => {
    const promise = createDocument
      .mutateAsync({ title: "Untitled" })
      .then((documentId) => navigate(`/documents/${documentId}`));

    toast.promise(promise, {
      loading: "Creating a new note....",
      success: "New note created!",
      error: "Failed to create a new note.",
    });
  };

  return (
    <div className="flex h-full flex-col items-center justify-center space-y-4">
      <img
        src="/empty.svg"
        alt="empty"
        height="300"
        width="300"
        className="h-auto dark:hidden"
      />
      <img
        src="/empty-dark.svg"
        alt="empty"
        height="300"
        width="300"
        className="hidden h-auto dark:block"
      />
      <h2 className="text-lg font-medium">Welcome to AnyWorkspace</h2>
      <Button onClick={onCreate}>
        <PlusCircle className="mr-2 h-4 w-4" />
        Create a note
      </Button>
    </div>
  );
};

export default DocumentsPage;
